[<img width="764" height="939" alt="image" src="https://github.com/user-attachments/assets/b401b0f5-637f-48fe-84ce-6eceddfa6570" />](https://github.com/user-attachments/assets/87e4596e-861f-4d38-b943-54d9d2fccf19)


The aim of this toolbox is to simulate an XInput Xbox 360 controller, allowing you to play games while learning how to code.   
Learning to code involves understanding concepts such as 2D arrays and images. I recommend buying an SSD1306 display to practise your coding skills.   
Since coding is also closely linked to colours, you can buy a 100-LED strip to experiment with RGB lighting.   
If you buy an ESP32, you might also want to have some fun building a small four-wheeled car using the L298N motor controller, which can also be used as a power source.     

# Hardware  
  
**Freenove Board**     
https://www.amazon.com.be/Freenove-Universal-Breakout-Raspberry-Switchable/dp/B0DT48T7CB  

**ESP32-S3**    
https://www.amazon.com.be/-/en/Freenove-ESP32-S3-Development-Dual-core-Microcontroller/dp/B0DHJZ1V81/  

**SSD1306 Display**     
https://www.amazon.com.be/s?k=ssd1306   

**5V WS2812B RGB LED Strip (1 m)**    
https://www.amazon.com.be/dp/B0D5B7HDPP   

**USB Power Supply for the LED Strip**   
You will need a USB power supply to power the LED strip connected to the Freenove board.  
https://www.amazon.com.be/dp/B0DNYGKXH5   

**Level Shifter**    
As you will most probably want to use microcontrollers that don't have Wi-Fi, a level shifter will help you connect and use them from a Godot/Unity application.    
https://www.amazon.com.be/s?k=level+shifter   
 


# Optional: Build a Small Car

If you want to take things a step further, you can build a small car using the following components:

**L298N Motor Controller Module**  
https://www.amazon.com.be/-/en/Controller-Stepper-Capacity-Regulator-Projects/dp/B0GXF8HB48

**Motors**  
https://www.amazon.com.be/-/en/3V-6V-Shaft-200RPM-intelligent-double/dp/B0FB3FLZ26/

# Optional: Add a Camera

Make your project even more fun by adding a camera.

**XIAO ESP32-S3 Sense**  
https://www.amazon.com.be/-/en/dp/B0C69FFVHH



# Instructions

Create a Wi-Fi hotspot named `uart.local` with the password `12345678`.

You can change these settings by editing `config.rs` before flashing the firmware.

Connect to the hotspot and send text or bytes through port `3615`.

The incoming data is interpreted as follows:

- **1024-byte array:** Sent to the SSD1306 screen.
- **300-byte array:** Sent to the LED strip.
- **5000-byte datagram:** Sent to the 200x200 monochrome e-paper screen, not UART.
- **Byte arrays divisible by 16:** Interpreted as blocks of timed instructions to be executed by the hardware.
- **Other data:** Forwarded to the device's UART pins and translated to check whether an Xbox 360 (X360) action is requested.

## Optional: 1.54-inch e-paper display

The startup image supports the **Waveshare 200x200 monochrome V2** panel
(SSD1681), not the V1 or three-color versions. Connect its V2.1 driver board:

| E-paper | ESP32-S3 |
| --- | --- |
| VCC | 3.3V |
| GND | GND |
| BUSY | GPIO1 |
| RST | GPIO2 |
| DC | GPIO42 |
| CS | GPIO41 |
| CLK | GPIO40 |
| DIN | GPIO39 |

These are GPIO numbers, not physical pin positions. GPIO39 through GPIO42 are
also JTAG pins; using them for the display prevents external JTAG debugging.
GPIO19/20 remain assigned to native USB, GPIO48 to the onboard LED, and
GPIO0/45 are boot strapping pins. GPIO35-37 can be used by octal flash/PSRAM.
Check your particular board's pinout before connecting.

After Wi-Fi obtains an IPv4 address, the display shows the address and UTC
time obtained from `pool.ntp.org`, plus an XInput command cheat sheet, then
enters deep sleep. Each pair is the on/off command (for stick axes, set to
the indicated value / reset to zero). `D` denotes the D-pad and `D0` its
neutral command; both 1310 and 2310 set it to neutral. `MENU>`/`MENU<` are
the right/left menu buttons; `LB`/`RB` and `LS`/`RS` are the side buttons and
stick clicks. `LX`, `LY`, `RX`, and `RY` are the left/right stick axes;
`+` and `-` mean 1.0 and -1.0. The lookup has a
10-second timeout; if it fails, the time is shown as `Unavailable` and the
reason is logged. The time is a one-time snapshot, not a running clock, and
NTP requires internet access (UDP port 123). The timestamp era is interpreted
as a date from 1970 through 2106. A later UDP image replaces this screen.
Set `EINK_STARTUP_ENABLED` to `false` in `src/config.rs`
when the display is not connected, or change the GPIO constants there.
Send an IPv4 UDP datagram containing exactly 5000 bytes to port 3615 to refresh
it: 200 rows of 25 bytes, left to right and top to bottom. Within each byte,
the most significant bit is the leftmost pixel; 0 is black and 1 is white.
For example, `0x7f` starts a row with one black pixel followed by seven white
pixels. The network must allow IPv4 fragmentation and reassembly.

Frames refresh as soon as possible, one at a time; up to two can wait in the
queue. Further frames are dropped with a serial warning. **Waveshare recommends
at least 180 seconds between full refreshes:** sending frames or restarting more
often is outside that recommendation and may shorten display life.

For troubleshooting, monitor the UART at 115200 baud and look for `[EINK]`
messages. A BUSY timeout or a missing BUSY transition indicates that the
display did not respond as expected; check power, wiring and panel version.
If the refresh cannot be confirmed, power off the display before leaving it
unattended. A completed BUSY cycle confirms a controller response, not the
contents of the visible image.

More details will be available in an upcoming tutorial.
