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
use embassy_futures::join::{join, join3};
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embedded_io_async::Write as AsyncWrite;
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_6X10};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Baseline, Text};
use esp_hal::{Async, Blocking};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use xesp32s3::debug::{self, DebugInfo, DEBUG, WIFI_STATUS, WifiStatus};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Ipv4Address, Runner, Stack, StackResources};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::control::{InResponse, OutResponse, Request, RequestType};
use embassy_usb::types::StringIndex;
use embassy_usb::{Builder, Config as UsbConfig};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, DriveMode, Level, Output, OutputConfig, Pin};
use esp_hal::ledc::{channel, timer, Ledc, LowSpeed, LSGlobalClkSource};
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::peripherals::{DMA_CH0, GPIO48, LEDC, RMT, SPI2};
use esp_hal::rmt::Rmt;
use esp_hal::rng::Rng;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config as UartConfig, UartTx};
use esp_hal::usb::otg::embassy_usb_device::{Config as OtgConfig, Driver};
use esp_hal::usb::otg::Usb;
use esp_radio::wifi::scan::ScanConfig;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, Interface, Password, Ssid, WifiController,
};
use esp_println::println as rprintln;
use esp_hal_smartled::{RmtSmartLeds, WS2812_TIMING, buffer_size, color_order};
use qrcodegen_no_heap::{QrCode, QrCodeEcc, Version};
use smart_leds_trait::{RGB8, SmartLedsWriteAsync};
use static_cell::StaticCell;
use xesp32s3::config::{
    DISPLAY_DEBUG_INFO, MDNS_HOST, SCREEN_SCL_GPIO, SCREEN_SDA_GPIO, STRIP_GPIO, STRIP_LED_COUNT,
    UART1_TX_GPIO, UART2_TX_GPIO, UART_BAUD_RATE, UDP_PORT, WIFI_PASSWORD, WIFI_SSID,
};
use xesp32s3::mdns::{self, MDNS_GROUP, MDNS_PORT};
use xesp32s3::integer_action_to_led_strip::{encode_strip_frame, is_led_command, latest_rgb_packet, IntegerActionToLedStrip, LedUpdate, STRIP_FRAME_BYTES};
use xesp32s3::mini_car_l298n::MiniCarCommand;
use xesp32s3::network::{process_udp, should_forward_udp, udp_led_commands, udp_mini_car_command, udp_rgb_packet, CONTROLLER};
use xesp32s3::screen::{screen_pixels, udp_screen_packet, SCREEN_FRAME_BYTES};
use xesp32s3::integer_delayer::IndexValueIntegerTimeMillisecondsDelayer;
use xesp32s3::xinput;
use xesp32s3::commands;
use xesp32s3::compact_xbox_int_parser;

const BUILD_INFO: &str = concat!(
    env!("XESP32S3_BUILD_DATE"),
    " ",
    env!("XESP32S3_GIT_COMMIT")
);

const MAX_UDP_PAYLOAD: usize = 1472;
type ForwardPacket = heapless::Vec<u8, MAX_UDP_PAYLOAD>;
static UART_PACKETS: Channel<CriticalSectionRawMutex, ForwardPacket, 4> = Channel::new();
static LED_COMMANDS: Channel<CriticalSectionRawMutex, LedUpdate, 32> = Channel::new();
static SCREEN_FRAME: Signal<CriticalSectionRawMutex, [u8; SCREEN_FRAME_BYTES]> = Signal::new();
static MINI_CAR_COMMAND: Signal<CriticalSectionRawMutex, MiniCarCommand> = Signal::new();

static DELAYER: embassy_sync::mutex::Mutex<CriticalSectionRawMutex, IndexValueIntegerTimeMillisecondsDelayer> =
    embassy_sync::mutex::Mutex::new(IndexValueIntegerTimeMillisecondsDelayer::new(0.0, 0.0));

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

    let mut pins = [
        Some(peripherals.GPIO1.degrade()), Some(peripherals.GPIO2.degrade()),
        Some(peripherals.GPIO3.degrade()), Some(peripherals.GPIO4.degrade()),
        Some(peripherals.GPIO5.degrade()), Some(peripherals.GPIO6.degrade()),
        Some(peripherals.GPIO7.degrade()), Some(peripherals.GPIO8.degrade()),
        Some(peripherals.GPIO9.degrade()), Some(peripherals.GPIO10.degrade()),
        Some(peripherals.GPIO11.degrade()), Some(peripherals.GPIO12.degrade()),
        Some(peripherals.GPIO13.degrade()), Some(peripherals.GPIO14.degrade()),
        Some(peripherals.GPIO15.degrade()), Some(peripherals.GPIO16.degrade()),
        Some(peripherals.GPIO17.degrade()), Some(peripherals.GPIO18.degrade()),
        Some(peripherals.GPIO21.degrade()), Some(peripherals.GPIO38.degrade()),
        Some(peripherals.GPIO39.degrade()), Some(peripherals.GPIO40.degrade()),
        Some(peripherals.GPIO41.degrade()), Some(peripherals.GPIO42.degrade()),
        Some(peripherals.GPIO45.degrade()), Some(peripherals.GPIO46.degrade()),
        Some(peripherals.GPIO47.degrade()),
    ];
    let strip_pin = take_configured_pin(&mut pins, STRIP_GPIO);
    let uart1_pin = take_configured_pin(&mut pins, UART1_TX_GPIO);
    let uart2_pin = take_configured_pin(&mut pins, UART2_TX_GPIO);
    let screen_sda = take_configured_pin(&mut pins, SCREEN_SDA_GPIO);
    let screen_scl = take_configured_pin(&mut pins, SCREEN_SCL_GPIO);
    let car_pin_10 = take_configured_pin(&mut pins, 10);
    let car_pin_11 = take_configured_pin(&mut pins, 11);
    let car_pin_12 = take_configured_pin(&mut pins, 12);
    let car_pin_13 = take_configured_pin(&mut pins, 13);
    let car_pin_17 = take_configured_pin(&mut pins, 17);
    let car_pin_18 = take_configured_pin(&mut pins, 18);

    spawner.spawn(mini_car_task(
        peripherals.LEDC, car_pin_10, car_pin_11, car_pin_12, car_pin_13,
        car_pin_17, car_pin_18,
    ).unwrap());

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

    let uart_config = UartConfig::default().with_baudrate(UART_BAUD_RATE);
    let uart1 = UartTx::new(peripherals.UART1, uart_config)
        .expect("UART1 configuration failed")
        .with_tx(uart1_pin)
        .into_async();
    let uart2 = UartTx::new(peripherals.UART2, uart_config)
        .expect("UART2 configuration failed")
        .with_tx(uart2_pin)
        .into_async();

    spawner.spawn(wifi_task(wifi_controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());
    spawner.spawn(udp_task(stack).unwrap());
    spawner.spawn(uart_forward_task(uart1, uart2).unwrap());
    spawner.spawn(mdns_task(stack).unwrap());
    spawner.spawn(delayed_integers_task().unwrap());
    spawner.spawn(led_task(peripherals.RMT, peripherals.GPIO48, peripherals.SPI2, peripherals.DMA_CH0, strip_pin).unwrap());

    // Higher I2C frequency (400 kHz) to overcome USB interrupt latency
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default()
            .with_frequency(Rate::from_khz(400))
    )
        .expect("I2C config")
        .with_sda(screen_sda)
        .with_scl(screen_scl);
    spawner.spawn(display_task(i2c, stack).unwrap());

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
    usb_config.composite_with_iads = false; //KEEP ON FALSE NOT TRUE
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

fn take_configured_pin(pins: &mut [Option<AnyPin<'static>>], gpio: u8) -> AnyPin<'static> {
    pins.iter_mut()
        .find(|pin| pin.as_ref().is_some_and(|pin| pin.number() == gpio))
        .and_then(Option::take)
        .expect("configured GPIO is unavailable or already assigned")
}

#[embassy_executor::task]
async fn mini_car_task(
    peripheral: LEDC<'static>,
    pin_10: AnyPin<'static>,
    pin_11: AnyPin<'static>,
    pin_12: AnyPin<'static>,
    pin_13: AnyPin<'static>,
    pin_17: AnyPin<'static>,
    pin_18: AnyPin<'static>,
) {
    let mut direction = [
        Output::new(pin_10, Level::Low, OutputConfig::default()),
        Output::new(pin_11, Level::Low, OutputConfig::default()),
        Output::new(pin_12, Level::Low, OutputConfig::default()),
        Output::new(pin_13, Level::Low, OutputConfig::default()),
    ];
    let mut ledc = Ledc::new(peripheral);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut timer = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    timer.configure(timer::config::Config {
        duty: timer::config::Duty::Duty8Bit,
        clock_source: timer::LSClockSource::APBClk,
        frequency: Rate::from_khz(20),
    }).expect("mini car PWM timer configuration failed");
    let mut motor_a = ledc.channel(channel::Number::Channel0, pin_17);
    let mut motor_b = ledc.channel(channel::Number::Channel1, pin_18);
    for motor in [&mut motor_a, &mut motor_b] {
        motor.configure(channel::config::Config {
            timer: &timer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        }).expect("mini car PWM channel configuration failed");
    }

    let mut active = false;
    loop {
        let command = if active {
            match with_timeout(Duration::from_secs(1), MINI_CAR_COMMAND.wait()).await {
                Ok(command) => command,
                Err(_) => {
                    motor_a.set_duty(0).expect("mini car PWM stop failed");
                    motor_b.set_duty(0).expect("mini car PWM stop failed");
                    for pin in &mut direction {
                        pin.set_level(Level::Low);
                    }
                    active = false;
                    rprintln!("[CAR] Command timeout; motors stopped");
                    continue;
                }
            }
        } else {
            MINI_CAR_COMMAND.wait().await
        };

        motor_a.set_duty(0).expect("mini car PWM update failed");
        motor_b.set_duty(0).expect("mini car PWM update failed");
        for (pin, high) in direction.iter_mut().zip([
            command.pin_10, command.pin_11, command.pin_12, command.pin_13,
        ]) {
            pin.set_level(if high { Level::High } else { Level::Low });
        }
        motor_a.set_duty(command.pin_17_value() * 100 / 9)
            .expect("mini car PWM update failed");
        motor_b.set_duty(command.pin_18_value() * 100 / 9)
            .expect("mini car PWM update failed");
        active = command.pin_17_value() != 0
            || command.pin_18_value() != 0
            || command.pin_10 || command.pin_11 || command.pin_12 || command.pin_13;
        rprintln!("[CAR] Applied {}", command);
    }
}

#[embassy_executor::task]
async fn led_task(
    rmt_peripheral: RMT<'static>,
    led_pin: GPIO48<'static>,
    spi_peripheral: SPI2<'static>,
    dma_channel: DMA_CH0<'static>,
    strip_pin: AnyPin<'static>,
) {
    let frequency = Rate::from_mhz(80);
    let rmt = Rmt::new(rmt_peripheral, frequency)
        .expect("LED RMT initialization failed")
        .into_async();
    let mut led = RmtSmartLeds::<{ buffer_size::<RGB8>(1) }, _, RGB8, color_order::Grb>::new(
        WS2812_TIMING,
        rmt.channel0,
        led_pin,
        frequency,
    )
    .expect("RGB LED configuration failed");
    let rx_buffer = esp_hal::dma_rx_buffer!(4).expect("RGB strip RX DMA buffer failed");
    let tx_buffer = esp_hal::dma_tx_buffer!(STRIP_FRAME_BYTES).expect("RGB strip TX DMA buffer failed");
    let mut strip = Spi::new(
        spi_peripheral,
        SpiConfig::default().with_frequency(Rate::from_khz(2500)),
    )
    .expect("RGB strip SPI configuration failed")
    .with_mosi(strip_pin)
    .with_dma(dma_channel)
    .with_buffers(rx_buffer, tx_buffer)
    .into_async();
    let startup_color = random_strip_color(&Rng::new());
    let mut strip_state = IntegerActionToLedStrip::new(startup_color);
    let mut strip_frame = [0; STRIP_FRAME_BYTES];
    encode_strip_frame(strip_state.state(), &mut strip_frame);
    strip
        .write_async(&strip_frame)
        .await
        .expect("RGB strip startup update failed");

    let rng = Rng::new();
    let mut previous = random_led_color(&rng, RGB8::new(0, 0, 0));
    led.write([previous].into_iter())
        .await
        .expect("RGB LED update failed");
    let mut next_led_update = Instant::now() + Duration::from_secs(1);
    let mut deferred_update = None;
    loop {
        let update = if let Some(update) = deferred_update.take() {
            update
        } else {
            match select(Timer::at(next_led_update), LED_COMMANDS.receive()).await {
                Either::First(()) => {
                    let color = random_led_color(&rng, previous);
                    led.write([color].into_iter())
                        .await
                        .expect("RGB LED update failed");
                    previous = color;
                    next_led_update = Instant::now() + Duration::from_secs(1);
                    continue;
                }
                Either::Second(update) => update,
            }
        };
        let changed = match update {
            LedUpdate::Integer(command) => match strip_state.handle_integer(command) {
                Ok(changed) => changed,
                Err(index) => {
                    rprintln!(
                        "LED command {} rejected: index {} exceeds strip length {}",
                        command, index, STRIP_LED_COUNT
                    );
                    false
                }
            },
            LedUpdate::Rgb(packet) => {
                let mut queued = core::iter::from_fn(|| LED_COMMANDS.try_receive().ok())
                    .take(LED_COMMANDS.len());
                let (packet, next) = latest_rgb_packet(packet, &mut queued);
                deferred_update = next;
                strip_state.handle_rgb_packet(&packet)
            }
        };
        if changed {
            encode_strip_frame(strip_state.state(), &mut strip_frame);
            strip
                .write_async(&strip_frame)
                .await
                .expect("RGB strip update failed");
        }
    }
}

fn random_strip_color(rng: &Rng) -> RGB8 {
    let colors = [
        RGB8::new(0, 0, 255),
        RGB8::new(255, 0, 0),
        RGB8::new(0, 255, 0),
        RGB8::new(255, 255, 0),
        RGB8::new(0, 255, 255),
        RGB8::new(255, 0, 255),
    ];
    let choice: u32 = rng.random();
    colors[choice as usize % colors.len()]
}

fn random_led_color(rng: &Rng, previous: RGB8) -> RGB8 {
    let random = rng.random();
    let mut color = RGB8::new(
        (random & 31) as u8,
        ((random >> 8) & 31) as u8,
        ((random >> 16) & 31) as u8,
    );
    if color == previous {
        color.r = (color.r + 1) % 32;
    }
    color
}

fn draw_qr_splash<D>(display: &mut D)
where
    D: DrawTarget<Color = BinaryColor>,
{
    const QR_URL: &str = "https://github.com/EloiStree/XESP32S3";
    const QR_AREA: i32 = 64;
    const QR_VERSION: Version = Version::new(3);

    let mut temp_buffer = [0; QR_VERSION.buffer_len()];
    let mut out_buffer = [0; QR_VERSION.buffer_len()];
    let qr = QrCode::encode_text(
        QR_URL,
        &mut temp_buffer,
        &mut out_buffer,
        QrCodeEcc::Low,
        QR_VERSION,
        QR_VERSION,
        None,
        false,
    )
    .expect("QR URL does not fit version 3");
    let scale = (QR_AREA / qr.size()).max(1);
    let offset = (QR_AREA - qr.size() * scale) / 2;

    let _ = Rectangle::new(Point::new(0, 0), Size::new(64, 64))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(display);

    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                let _ = Rectangle::new(
                    Point::new(offset + x * scale, offset + y * scale),
                    Size::new(scale as u32, scale as u32),
                )
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                .draw(display);
            }
        }
    }

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    for (label, y) in [("SCAN", 8), ("FOR", 20), ("SOURCE", 32), ("CODE", 44)] {
        let _ = Text::with_baseline(label, Point::new(70, y), style, Baseline::Top).draw(display);
    }
}

#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>) {
    WIFI_STATUS.signal(WifiStatus::WaitingForHotspot);

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
                WIFI_STATUS.signal(WifiStatus::Connected);
                rprintln!("Wi-Fi connected to {}", WIFI_SSID);
                let _ = controller.wait_for_disconnect_async().await;
                rprintln!("Wi-Fi disconnected");
                WIFI_STATUS.signal(WifiStatus::WaitingForHotspot);
            }
            Err(e) => {
                WIFI_STATUS.signal(WifiStatus::WaitingForHotspot);
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

/// SSD1306 128x64 on I2C0 (address 0x3C).
#[embassy_executor::task]
async fn display_task(i2c: I2c<'static, Blocking>, stack: Stack<'static>) {
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    if display.init().is_err() {
        rprintln!("SSD1306 init failed");
        return;
    }

    draw_qr_splash(&mut display);
    let _ = display.flush();
    Timer::after(Duration::from_secs(4)).await;

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let network_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
    let mut info = DebugInfo::default();
    let mut wifi_status = WifiStatus::WaitingForHotspot;
    let mut showing_frame = false;

    loop {
        if !showing_frame {
            display.clear_buffer();
            match wifi_status {
                WifiStatus::WaitingForHotspot => {
                    let mut draw = |y: i32, text: &str| {
                        let _ = Text::with_baseline(text, Point::new(0, y), style, Baseline::Top)
                            .draw(&mut display);
                    };
                    draw(0, "Wi-Fi not connected");
                    draw(11, "Create a hotspot:");

                    let mut ssid = heapless::String::<40>::new();
                    let _ = write!(ssid, "SSID: {}", WIFI_SSID);
                    draw(22, &ssid);

                    let mut password = heapless::String::<40>::new();
                    let _ = write!(password, "Password: {}", WIFI_PASSWORD);
                    draw(33, &password);

                    draw(44, "Waiting for Wi-Fi...");
                    draw(54, BUILD_INFO);
                }
                WifiStatus::Connected => {
                    let mut network = heapless::String::<48>::new();
                    let _ = write!(
                        network,
                        "W:{} M:{}.local:{}",
                        WIFI_SSID, MDNS_HOST, UDP_PORT
                    );
                    let _ = Text::with_baseline(
                        &network,
                        Point::new(0, 0),
                        network_style,
                        Baseline::Top,
                    )
                    .draw(&mut display);
                    let mut draw = |y: i32, text: &str| {
                        let _ = Text::with_baseline(text, Point::new(0, y), style, Baseline::Top)
                            .draw(&mut display);
                    };
                    let mut ip = heapless::String::<24>::new();
                    if let Some(config) = stack.config_v4() {
                        let _ = write!(ip, "IP: {}", config.address.address());
                    } else {
                        let _ = ip.push_str("IP: waiting...");
                    }
                    draw(11, &ip);
                    if DISPLAY_DEBUG_INFO {
                        draw(22, &info.state);
                        draw(33, &info.axes);
                    }
                    draw(54, BUILD_INFO);
                }
            }
            if let Err(error) = display.flush() {
                rprintln!("SSD1306 status update failed: {:?}", error);
            }
        }

        match select(
            SCREEN_FRAME.wait(),
            select(
                select(DEBUG.wait(), WIFI_STATUS.wait()),
                Timer::after(Duration::from_secs(1)),
            ),
        )
        .await
        {
            Either::First(frame) => {
                showing_frame = true;
                display.clear_buffer();
                for (x, y, on) in screen_pixels(&frame) {
                    display.set_pixel(x, y, on);
                }
                if let Err(error) = display.flush() {
                    rprintln!("SSD1306 frame update failed: {:?}", error);
                }
            }
            Either::Second(Either::First(Either::First(updated_info))) => info = updated_info,
            Either::Second(Either::First(Either::Second(updated_status))) => wifi_status = updated_status,
            Either::Second(Either::Second(())) => {}
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}

#[embassy_executor::task]
async fn uart_forward_task(mut uart1: UartTx<'static, Async>, mut uart2: UartTx<'static, Async>) {
    loop {
        let packet = UART_PACKETS.receive().await;
        let (result1, result2) = join(uart1.write_all(&packet), uart2.write_all(&packet)).await;
        if let Err(error) = result1 {
            rprintln!("UART1 forwarding failed: {:?}", error);
        }
        if let Err(error) = result2 {
            rprintln!("UART2 forwarding failed: {:?}", error);
        }
    }
}

#[embassy_executor::task]
async fn delayed_integers_task() {
    rprintln!("[DELAYED_INTEGERS_TASK] Started");

    loop {
        let ready = {
            let mut delayer = DELAYER.lock().await;
            delayer.check_for_integer_to_execute()
        };
        let delay_ms = if ready.is_empty() { 10 } else { 1 };

        for delayed_value in ready {
            let index = delayed_value.m_index;
            let value = delayed_value.m_integer;
            rprintln!("[DELAYED] Ready: index={}, value={}", index, value);

            if is_led_command(value) {
                LED_COMMANDS.send(LedUpdate::Integer(value)).await;
                continue;
            }

            // Apply the delayed integer to the Xbox controller
            let mut state = CONTROLLER.lock().await;

            // Capture state before for verification
            let buttons_before = state.buttons;
            let dpad_before = state.dpad;
            let left_x_before = state.left_x;
            let right_trigger_before = state.right_trigger;

            // Convert the delayed value to a command and execute it
            let executed = if let Ok(command) = u16::try_from(value) {
                rprintln!("[DELAYED] Trying u16 command: {}", command);
                if commands::execute(&mut state, command) {
                    rprintln!("[DELAYED] ✓ Applied Xbox command (u16): value={}", value);
                    true
                } else {
                    rprintln!("[DELAYED] Fallback to compact parser for: {}", value);
                    compact_xbox_int_parser::execute(&mut state, value);
                    rprintln!("[DELAYED] ✓ Applied via compact parser: value={}", value);
                    true
                }
            } else {
                rprintln!("[DELAYED] Value too large for u16 ({}), using compact parser", value);
                compact_xbox_int_parser::execute(&mut state, value);
                rprintln!("[DELAYED] ✓ Applied via compact parser: value={}", value);
                true
            };

            // Verify state changed
            if executed {
                let buttons_after = state.buttons;
                let dpad_after = state.dpad;
                let left_x_after = state.left_x;
                let right_trigger_after = state.right_trigger;

                if buttons_before != buttons_after {
                    rprintln!("[DELAYED] ✓ Buttons changed: {:#010x} → {:#010x}", buttons_before, buttons_after);
                } else if core::mem::discriminant(&dpad_before) != core::mem::discriminant(&dpad_after) {
                    rprintln!("[DELAYED] ✓ DPad changed: {:?} → {:?}", dpad_before, dpad_after);
                } else if (left_x_before - left_x_after).abs() > 0.01 {
                    rprintln!("[DELAYED] ✓ Left X changed: {} → {}", left_x_before, left_x_after);
                } else if (right_trigger_before - right_trigger_after).abs() > 0.01 {
                    rprintln!("[DELAYED] ✓ Right trigger changed: {} → {}", right_trigger_before, right_trigger_after);
                } else {
                    rprintln!("[DELAYED] ⚠ State unchanged (command may have failed or been a no-op)");
                }
            }
        }
        
        // Small delay to avoid busy-waiting
        Timer::after(Duration::from_millis(delay_ms)).await;
    }
}

#[embassy_executor::task]
async fn udp_task(stack: Stack<'static>) {
    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut rx_buffer = [0u8; 8192];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_buffer = [0u8; 64];

    stack.wait_config_up().await;
    if let Some(cfg) = stack.config_v4() {
        rprintln!("IP {} - UDP {}", cfg.address, UDP_PORT);
    }

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(UDP_PORT).expect("UDP bind failed");

    let mut buf = [0u8; MAX_UDP_PAYLOAD];
    loop {
        let (len, _from) = match socket.recv_from(&mut buf).await {
            Ok(packet) => packet,
            Err(error) => {
                rprintln!("UDP receive failed: {:?}", error);
                continue;
            }
        };
        let packet = &buf[..len];
        
        rprintln!("[UDP] ✓ Packet received: {} bytes", len);

        if let Some(command) = udp_mini_car_command(packet) {
            MINI_CAR_COMMAND.signal(command);
        } else if packet.get(..4).is_some_and(|header| header.eq_ignore_ascii_case(b"CAR:")) {
            rprintln!("[CAR] Invalid command: expected CAR:AADDDD (10 bytes)");
        }

        if let Some(frame) = udp_screen_packet(packet) {
            SCREEN_FRAME.signal(*frame);
        }

        if let Some(rgb) = udp_rgb_packet(packet) {
            LED_COMMANDS.send(LedUpdate::Rgb(*rgb)).await;
        } else {
            // Check if UDP payload is divisible by 16 and process as delayed integers
            let mut delayer = DELAYER.lock().await;
            let (is_iid_format, parsed_count) = delayer.append_value_if_dividable_by_16_and_not_1024(packet);
            
            if is_iid_format {
                rprintln!("[UDP] ✓ IID format packet received and queued ({} items, {} total pending)", 
                    parsed_count, delayer.queue_length());
            } else {
                rprintln!("[UDP] ✗ Not IID format - will process as regular packet");
            }
            drop(delayer);

            let mut state = CONTROLLER.lock().await;
            let applied = process_udp(&mut state, packet);
            if DISPLAY_DEBUG_INFO {
                DEBUG.signal(debug::describe(packet, applied, &state));
            }
            drop(state);

            for command in udp_led_commands(packet) {
                LED_COMMANDS.send(LedUpdate::Integer(command)).await;
            }
        }

        if should_forward_udp(packet) {
            let packet = ForwardPacket::from_slice(packet).expect("UDP packet exceeds buffer");
            if UART_PACKETS.try_send(packet).is_err() {
                rprintln!("UART forwarding queue full; UDP packet dropped");
            }
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
