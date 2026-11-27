//! Images for the Waveshare 1.54-inch monochrome V2 (SSD1681) panel.
//!
//! Uses the panel's built-in full-refresh waveform, not the waveform for V1 or
//! three-color panels.
//! 
//! 
//! Display	ESP32-S3
// BUSY / RST / DC	GPIO1 / 2 / 42
// CS / CLK / DIN	GPIO41 / 40 / 39
// VCC / GND	3.3V / GND

use core::fmt::Write;
use embassy_net::dns::DnsQueryType;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpEndpoint, Stack};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_hal::delay::Delay;
use esp_hal::gpio::{AnyPin, Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::rng::Rng;
use esp_hal::spi::{master::Spi, Error as SpiError};
use esp_hal::Blocking;
use esp_println::println;
use crate::ntp;
use crate::screen::{eink_startup_image, EINK_FRAME_BYTES};

const BUSY_TIMEOUT: Duration = Duration::from_secs(15);
const NTP_TIMEOUT: Duration = Duration::from_secs(10);
static FRAMES: Channel<CriticalSectionRawMutex, [u8; EINK_FRAME_BYTES], 2> = Channel::new();

/// Queues a complete row-major, MSB-first image for the e-paper display.
pub fn queue_frame(frame: &[u8; EINK_FRAME_BYTES]) -> bool {
    FRAMES.try_send(*frame).is_ok()
}

#[derive(Debug)]
enum Error {
    Spi(SpiError),
    BusyTimeout(&'static str),
    RefreshNotStarted,
}

impl From<SpiError> for Error {
    fn from(error: SpiError) -> Self {
        Self::Spi(error)
    }
}

struct Display {
    spi: Spi<'static, Blocking>,
    busy: Input<'static>,
    reset: Output<'static>,
    dc: Output<'static>,
    cs: Output<'static>,
}

impl Display {
    fn write(&mut self, data_mode: bool, bytes: &[u8]) -> Result<(), Error> {
        if data_mode {
            self.dc.set_high();
        } else {
            self.dc.set_low();
        }
        self.cs.set_low();
        let result = self.spi.write(bytes);
        self.cs.set_high();
        result.map_err(Error::from)
    }

    fn command(&mut self, command: u8, data: &[u8]) -> Result<(), Error> {
        self.write(false, &[command])?;
        if !data.is_empty() {
            self.write(true, data)?;
        }
        Ok(())
    }

    async fn wait_ready(&self, phase: &'static str) -> Result<(), Error> {
        let deadline = Instant::now() + BUSY_TIMEOUT;
        while self.busy.is_high() {
            if Instant::now() >= deadline {
                return Err(Error::BusyTimeout(phase));
            }
            Timer::after_millis(10).await;
        }
        Ok(())
    }

    async fn wait_for_refresh(&self) -> Result<(), Error> {
        let deadline = Instant::now() + Duration::from_secs(1);
        while !self.busy.is_high() {
            if Instant::now() >= deadline {
                return Err(Error::RefreshNotStarted);
            }
            Timer::after_millis(1).await;
        }
        self.wait_ready("refresh").await
    }

    fn set_cursor(&mut self) -> Result<(), Error> {
        self.command(0x4e, &[0])?;
        self.command(0x4f, &[0, 0])
    }

    async fn show_frame(&mut self, frame: &[u8; EINK_FRAME_BYTES]) -> Result<(), Error> {
        self.reset.set_high();
        Timer::after_millis(20).await;
        self.reset.set_low();
        // The V2.1 board can power off if reset is held low for too long.
        Delay::new().delay_millis(2);
        self.reset.set_high();
        Timer::after_millis(20).await;
        if self.busy.is_high() {
            println!("[EINK] BUSY high after hardware reset; sending software reset");
        }
        self.command(0x12, &[])?;
        Timer::after_millis(10).await;
        self.wait_ready("software reset").await?;

        self.command(0x01, &[199, 0, 0])?;
        self.command(0x3c, &[0x05])?;
        self.command(0x18, &[0x80])?;
        self.command(0x11, &[0x03])?;
        self.command(0x44, &[0, 24])?;
        self.command(0x45, &[0, 0, 199, 0])?;

        // Initialize both RAM planes; there is no previous image in controller RAM after boot.
        self.set_cursor()?;
        self.command(0x26, &[])?;
        let white = [0xff; 25];
        for _ in 0..200 {
            self.write(true, &white)?;
        }
        self.set_cursor()?;
        self.command(0x24, &[])?;
        for row in frame.chunks_exact(25) {
            self.write(true, row)?;
        }
        self.command(0x22, &[0xf7])?;
        self.command(0x20, &[])?;
        println!("[EINK] Refresh command sent; waiting for BUSY");
        self.wait_for_refresh().await?;
        self.command(0x10, &[0x01])?;
        Timer::after_millis(100).await;
        Ok(())
    }
}

async fn fetch_utc_seconds(stack: Stack<'_>) -> Result<u32, &'static str> {
    let addresses = stack.dns_query("pool.ntp.org", DnsQueryType::A).await
        .map_err(|_| "DNS lookup failed")?;
    let address = *addresses.first().ok_or("DNS returned no address")?;
    let endpoint = IpEndpoint::new(address, 123);

    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut rx_buffer = [0; 512];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_buffer = [0; ntp::NTP_PACKET_BYTES];
    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(0).map_err(|_| "UDP bind failed")?;

    let rng = Rng::new();
    let mut nonce = [0; 8];
    nonce[..4].copy_from_slice(&rng.random().to_be_bytes());
    nonce[4..].copy_from_slice(&rng.random().to_be_bytes());
    if nonce == [0; 8] {
        nonce[7] = 1;
    }
    socket.send_to(&ntp::request(nonce), endpoint).await
        .map_err(|_| "NTP request failed")?;

    let mut response = [0; 512];
    loop {
        let (len, metadata) = socket.recv_from(&mut response).await
            .map_err(|_| "NTP response was too large")?;
        if metadata.endpoint != endpoint {
            println!("[EINK] Ignoring NTP reply from unexpected endpoint");
            continue;
        }
        if let Some(seconds) = ntp::response_seconds_of_day(&response[..len], nonce) {
            return Ok(seconds);
        }
        println!("[EINK] Ignoring invalid NTP reply");
    }
}

#[embassy_executor::task]
pub async fn startup_task(
    stack: Stack<'static>,
    spi: Spi<'static, Blocking>,
    busy: AnyPin<'static>,
    reset: AnyPin<'static>,
    dc: AnyPin<'static>,
    cs: AnyPin<'static>,
) {
    println!("[EINK] Starting 200x200 V2 display");
    let mut display = Display {
        spi,
        busy: Input::new(busy, InputConfig::default().with_pull(Pull::Up)),
        reset: Output::new(reset, Level::High, OutputConfig::default()),
        dc: Output::new(dc, Level::Low, OutputConfig::default()),
        cs: Output::new(cs, Level::High, OutputConfig::default()),
    };
    println!("[EINK] Waiting for IPv4 address");
    let config = loop {
        if let Some(config) = stack.config_v4() {
            break config;
        }
        Timer::after_millis(500).await;
    };
    let mut ip = heapless::String::<15>::new();
    write!(&mut ip, "{}", config.address.address())
        .expect("IPv4 address must fit in 15 characters");

    let mut utc = heapless::String::<11>::new();
    let seconds = match with_timeout(NTP_TIMEOUT, fetch_utc_seconds(stack)).await {
        Ok(Ok(seconds)) => Some(seconds),
        Ok(Err(reason)) => {
            println!("[EINK] UTC time unavailable: {}", reason);
            None
        }
        Err(_) => {
            println!("[EINK] UTC time lookup timed out");
            None
        }
    };
    if let Some(seconds) = seconds {
        write!(&mut utc, "{:02}:{:02}:{:02}", seconds / 3600, seconds / 60 % 60, seconds % 60)
            .expect("UTC time must fit in eight characters");
        println!("[EINK] Internet UTC time: {}", utc);
    } else {
        utc.push_str("Unavailable").expect("status must fit in UTC buffer");
    }
    let startup = eink_startup_image(&ip, &utc);
    match display.show_frame(startup.data()).await {
        Ok(()) => println!("[EINK] Startup image refreshed"),
        Err(error) => println!("[EINK] Startup refresh failed: {:?}", error),
    }
    loop {
        let frame = FRAMES.receive().await;
        match display.show_frame(&frame).await {
            Ok(()) => println!("[EINK] Refresh completed; sleep command sent"),
            Err(Error::BusyTimeout(phase)) => {
                println!("[EINK] BUSY stayed high during {}; check power and wiring", phase)
            }
            Err(Error::RefreshNotStarted) => {
                println!("[EINK] BUSY never went high after refresh; check panel version, power and wiring")
            }
            Err(Error::Spi(error)) => println!("[EINK] SPI error: {:?}", error),
        }
    }
}