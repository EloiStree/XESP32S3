#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_futures::select::{select, Either};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use esp_hal::Blocking;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use xesp32s3::debug::{self, DebugInfo, DEBUG, IP};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Ipv4Address, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::control::{InResponse, OutResponse, Request, RequestType};
use embassy_usb::types::StringIndex;
use embassy_usb::{Builder, Config as UsbConfig};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::usb::otg::embassy_usb_device::{Config as OtgConfig, Driver};
use esp_hal::usb::otg::Usb;
use esp_radio::wifi::scan::ScanConfig;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, Interface, Password, Ssid, WifiController,
};
use esp_println::println as rprintln;
use static_cell::StaticCell;
use xesp32s3::mdns::{self, MDNS_GROUP, MDNS_PORT};
use xesp32s3::network::{process_udp, CONTROLLER, MDNS_HOST, UDP_PORT, WIFI_PASSWORD, WIFI_SSID};
use xesp32s3::xinput;

/// Answers the vendor control requests the Windows Xbox 360 driver issues at enumeration.
struct XInputHandler {
    xsm3_string: StringIndex,
}

impl embassy_usb::Handler for XInputHandler {
    fn get_string(&mut self, index: StringIndex, _lang_id: u16) -> Option<&str> {
        (index == self.xsm3_string).then_some(xinput::XSM3_STRING)
    }

    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if req.request_type != RequestType::Vendor {
            return None;
        }
        rprintln!(
            "USB vendor IN: req={:#04x} value={:#06x} index={} len={}",
            req.request,
            req.value,
            req.index,
            req.length
        );
        // A zero-length success is what working Xbox 360 emulators answer with.
        Some(InResponse::Accepted(&buf[..0]))
    }

    fn control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        if req.request_type != RequestType::Vendor {
            return None;
        }
        rprintln!("USB vendor OUT: req={:#04x} value={:#06x} data={:02x?}", req.request, req.value, data);
        Some(OutResponse::Accepted)
    }
}

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", panic_info);
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: StaticCell<$t> = StaticCell::new();
        STATIC_CELL.uninit().write($val)
    }};
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    // Wi-Fi station + embassy-net stack
    let wifi_controller = WifiController::new(peripherals.WIFI, Default::default())
        .expect("Failed to initialize Wi-Fi controller");
    let interface = Interface::station();

    let seed = Rng::new().random() as u64;
    let (stack, runner) = embassy_net::new(
        interface,
        embassy_net::Config::dhcpv4(Default::default()),
        mk_static!(StackResources<5>, StackResources::new()),
        seed,
    );

    spawner.spawn(wifi_task(wifi_controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());
    spawner.spawn(udp_task(stack).unwrap());
    spawner.spawn(mdns_task(stack).unwrap());

    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .expect("I2C config")
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);
    spawner.spawn(display_task(i2c).unwrap());

    // Native USB (second USB-C port, GPIO19 = D-, GPIO20 = D+) as an Xbox 360 controller
    let usb = Usb::new_fs(peripherals.USB_FS, peripherals.GPIO20, peripherals.GPIO19);
    let ep_out_buffer = mk_static!([u8; 256], [0; 256]);
    let driver = Driver::new(usb, ep_out_buffer, OtgConfig::default());

    let mut usb_config = UsbConfig::new(xinput::VENDOR_ID, xinput::PRODUCT_ID);
    usb_config.manufacturer = Some("Microsoft");
    usb_config.product = Some("Controller");
    usb_config.serial_number = Some("08FEC93");
    usb_config.device_release = xinput::DEVICE_RELEASE;
    usb_config.device_class = 0xFF;
    usb_config.device_sub_class = 0xFF;
    usb_config.device_protocol = 0xFF;
    usb_config.composite_with_iads = false;
    usb_config.max_power = 500;
    usb_config.max_packet_size_0 = 64;

    let mut builder = Builder::new(
        driver,
        usb_config,
        mk_static!([u8; 256], [0; 256]),
        mk_static!([u8; 256], [0; 256]),
        &mut [],
        mk_static!([u8; 256], [0; 256]),
    );

    let xsm3_string = builder.string();
    let mut function = builder.function(0xFF, 0xFF, 0xFF);

    // Interface 0: controller data (input reports + rumble/LED output)
    let (mut ep_in, mut ep_out) = {
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(
            xinput::IFACE_CLASS,
            xinput::IFACE_SUBCLASS,
            xinput::IFACE_PROTOCOL,
            None,
        );
        alt.descriptor(0x21, &xinput::CLASS_DESCRIPTOR);
        (
            alt.endpoint_interrupt_in(None, 32, 4),
            alt.endpoint_interrupt_out(None, 32, 8),
        )
    };

    // Interface 1: audio (unused, present to match a genuine pad)
    let _audio = {
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(0xFF, 0x5D, 0x03, None);
        alt.descriptor(0x21, &xinput::AUDIO_DESCRIPTOR);
        (
            alt.endpoint_interrupt_in(None, 32, 2),
            alt.endpoint_interrupt_out(None, 32, 4),
            alt.endpoint_interrupt_in(None, 32, 64),
            alt.endpoint_interrupt_out(None, 32, 16),
        )
    };

    // Interface 2: plugin module (unused)
    let _plugin = {
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(0xFF, 0x5D, 0x02, None);
        alt.descriptor(0x21, &xinput::PLUGIN_DESCRIPTOR);
        alt.endpoint_interrupt_in(None, 32, 16)
    };

    // Interface 3: security (no endpoints, vendor descriptor type 0x41 + XSM3 string)
    {
        let mut interface = function.interface();
        let mut alt = interface.alt_setting(0xFF, 0xFD, 0x13, Some(xsm3_string));
        alt.descriptor(0x41, &xinput::SECURITY_DESCRIPTOR);
    }
    drop(function);

    builder.handler(mk_static!(XInputHandler, XInputHandler { xsm3_string }));

    let mut usb_device = builder.build();

    let report_loop = async {
        let mut sent = 0u32;
        loop {
            ep_in.wait_enabled().await;
            rprintln!("USB host connected");
            loop {
                let report = xinput::encode(&*CONTROLLER.lock().await);
                if ep_in.write(&report).await.is_err() {
                    break;
                }
                sent += 1;
                if sent == 1 || sent % 500 == 0 {
                    rprintln!("USB IN reports sent: {}", sent);
                }
                Timer::after(Duration::from_millis(4)).await;
            }
        }
    };

    // Rumble / LED packets from the host are read and ignored.
    let host_to_device = async {
        let mut buf = [0u8; 32];
        loop {
            ep_out.wait_enabled().await;
            while let Ok(n) = ep_out.read(&mut buf).await {
                rprintln!("USB OUT {} bytes: {:02x?}", n, &buf[..n]);
            }
        }
    };

    join3(usb_device.run(), report_loop, host_to_device).await;
    unreachable!()
}

#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>) {
    let config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(Ssid::try_from(WIFI_SSID).expect("invalid SSID"))
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                Password::try_from(WIFI_PASSWORD).expect("invalid password"),
            )),
    );
    controller.set_config(&config).expect("invalid Wi-Fi config");

    loop {
        match controller.connect_async().await {
            Ok(_) => {
                rprintln!("Wi-Fi connected to {}", WIFI_SSID);
                let _ = controller.wait_for_disconnect_async().await;
                rprintln!("Wi-Fi disconnected");
            }
            Err(e) => {
                rprintln!("Wi-Fi connect failed: {:?}", e);
                match controller.scan_async(&ScanConfig::default()).await {
                    Ok(aps) => {
                        rprintln!("Visible networks: {}", aps.len());
                        for ap in aps.iter().take(15) {
                            rprintln!("  {:?}", ap);
                        }
                    }
                    Err(e) => rprintln!("Scan failed: {:?}", e),
                }
            }
        }
        Timer::after(Duration::from_secs(2)).await;
    }
}

/// SSD1306 128x64 on I2C0 (SDA GPIO8, SCL GPIO9, address 0x3C).
#[embassy_executor::task]
async fn display_task(i2c: I2c<'static, Blocking>) {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    if display.init().is_err() {
        rprintln!("SSD1306 init failed");
        return;
    }

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let mut ip = debug::Line::new();
    let _ = ip.push_str("waiting for Wi-Fi");
    let mut info = DebugInfo::default();

    loop {
        display.clear_buffer();
        let mut draw = |y: i32, text: &str| {
            let _ = Text::with_baseline(text, Point::new(0, y), style, Baseline::Top).draw(&mut display);
        };

        let mut l = heapless::String::<32>::new();
        let _ = write!(l, "{}:{}", ip, UDP_PORT);
        draw(0, &l);
        let mut rx = heapless::String::<32>::new();
        let _ = write!(rx, "RX {}", info.received);
        draw(11, &rx);
        let mut ok = heapless::String::<32>::new();
        let _ = write!(ok, "applied: {}", info.applied);
        draw(22, &ok);
        draw(33, &info.state);
        draw(44, &info.axes);
        let _ = display.flush();

        match select(DEBUG.wait(), IP.wait()).await {
            Either::First(i) => info = i,
            Either::Second(i) => ip = i,
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}

#[embassy_executor::task]
async fn udp_task(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut rx_buffer = [0u8; 512];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_buffer = [0u8; 64];

    stack.wait_config_up().await;
    if let Some(cfg) = stack.config_v4() {
        rprintln!("IP {} - UDP {}", cfg.address, UDP_PORT);
        let mut line = debug::Line::new();
        let _ = write!(line, "{}", cfg.address.address());
        IP.signal(line);
    }

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(UDP_PORT).expect("UDP bind failed");

    let mut buf = [0u8; 512];
    loop {
        if let Ok((len, _from)) = socket.recv_from(&mut buf).await {
            let mut state = CONTROLLER.lock().await;
            let applied = process_udp(&mut state, &buf[..len]);
            DEBUG.signal(debug::describe(&buf[..len], applied, &state));
        }
    }
}

#[embassy_executor::task]
async fn mdns_task(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buffer = [0u8; 512];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buffer = [0u8; 256];

    stack.wait_config_up().await;
    let group = Ipv4Address::new(MDNS_GROUP[0], MDNS_GROUP[1], MDNS_GROUP[2], MDNS_GROUP[3]);
    let _ = stack.join_multicast_group(group);

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(MDNS_PORT).expect("mDNS bind failed");

    let target = IpEndpoint::new(IpAddress::Ipv4(group), MDNS_PORT);
    let mut buf = [0u8; 512];
    let mut out = [0u8; 128];

    loop {
        let Ok((len, _from)) = socket.recv_from(&mut buf).await else {
            continue;
        };
        if !mdns::is_query_for(&buf[..len], MDNS_HOST) {
            continue;
        }
        let Some(cfg) = stack.config_v4() else {
            continue;
        };
        if let Some(n) = mdns::build_answer(&mut out, MDNS_HOST, cfg.address.address().octets()) {
            let _ = socket.send_to(&out[..n], target).await;
        }
    }
}
