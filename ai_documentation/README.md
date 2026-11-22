# XESP32S3

Test that it works:    
https://hardwaretester.com/gamepad   

**Vibe-coded in Rust: An Xbox 360 controller over Wi-Fi (UDP).**

Did you know that the USB ports on the ESP32-S3 can be used to simulate USB input devices?     
[📼 Here's the video that inspired me.](https://www.youtube.com/watch?v=M6fpiwUEPms&start=4)      
I didn't know. So naturally, I had to try it.   
<img width="1149" height="717" alt="XESP32S3 project" src="https://github.com/user-attachments/assets/aee36c81-277c-4891-bf6a-d8344a5991d6" />    

For the first time in my developer life, I 100% vibe-coded a tool.   
I'm not particularly good at Rust. I had no idea where to start, and I knew there wouldn't be much documentation available on this specific topic.    
But I really, really needed this tool.    

So I gave AI my S2W standard, explained what I wanted, and let it work for about two hours.      
And it fucking did it. 😜🔥      

We are doomed.  

All jokes aside, now that the tool exists, it's time for me to read through the code and figure out what It actually built.   
Hopefully, I'll understand most of it!     
**Feel free to use it:**
[GitHub – XESP32S3](https://github.com/eloistree/XESP32S3)

If you prefer the good old UART + Arduino Leonardo approach, here's my previous project:    
https://github.com/EloiStree/2023_06_21_arduino_to_xbox_360    

For keyboard, mouse, and MIDI projects, I personally prefer CircuitPython. It's much easier to understand and modify, even for younger developers:    
https://github.com/eloistree/s2w   

Hope it helps someone! 🚀   

⚠️ You still need an adapter like Brook, or use Xbox Remove Play, if you want to use it on your Xbox.⚠️    



I am using a Freenove ESP32S3 WROOM 1:      
[<img width="1500" height="1500" alt="image" src="https://github.com/user-attachments/assets/36fb02d5-fa90-43e2-83b4-153c7dd3dd56" />](https://store.freenove.com/products/fnk0099)      
https://store.freenove.com/products/fnk0099

The built-in WS2812 RGB LED on GPIO48 changes to a random, low-brightness color every second.
At startup, every LED on the WS2812B strip on GPIO14 is set to the same randomly
chosen full-intensity blue, red, green, yellow, cyan, or purple. Set
`STRIP_LED_COUNT` in [`src/config.rs`](src/config.rs) to the number of LEDs on the
strip (1-100, 100 by default). UDP LED commands can change individual LEDs without
changing the onboard LED. The strip uses SPI2 and DMA channel 0 for uninterrupted
transmission; the onboard LED continues to use RMT.
Connect GPIO14 to the strip's data input (DIN), use a suitable 5 V power supply, and
connect the strip and ESP32 grounds together. A 3.3 V-to-5 V logic level shifter is
recommended for reliable data. Check the strip's labels and data-direction arrows
instead of relying on wire colors. Size the power supply and wiring for the strip's
maximum current; 100 LEDs may draw up to approximately 6 A at full white.

With a SSD1306 I2C for debugging:       
[<img width="471" height="463" alt="image" src="https://github.com/user-attachments/assets/7e156472-982d-42bb-a6eb-1c7dad8b81cf" />](https://www.amazon.com.be/-/en/Fasizi-Serial-128x64-Display-Arduino/dp/B09Z27KV42)     
https://www.amazon.com.be/-/en/Fasizi-Serial-128x64-Display-Arduino/dp/B09Z27KV42


-------------------

# Configuration

Edit network settings, strip LED count, and the GPIO assignments in
[`src/config.rs`](src/config.rs), then rebuild and flash the firmware. Change each
GPIO constant to an available pin; assignments must be unique. Supported choices are
GPIO1-18, GPIO21, GPIO38-42 and GPIO45-47. Other pins are excluded because they are
reserved by the firmware or may be used for USB, flash, PSRAM or serial logging.
Check your board's peripherals before choosing a pin. GPIO3, GPIO45 and GPIO46 are
strapping pins and require special care at reset.

UDP datagrams are also forwarded unchanged to UART1 TX on GPIO4 and UART2 TX on GPIO5
(9600 baud, 8-N-1 by default), except for 300-byte LED packets and 1024-byte packets.
Connect the receiving devices' grounds to the ESP32's ground and use 3.3 V-compatible UART inputs.
Controller and LED commands in 1024-byte datagrams are still processed. The forwarding path accepts
unfragmented UDP payloads up to 1472 bytes; IPv4 fragmentation is not supported.
UART throughput is limited, so sustained traffic or bursts can cause packets to be
dropped; a full forwarding queue is reported in the serial log.

A UDP datagram of exactly 1024 bytes also updates the 128x64 SSD1306 screen.
Send eight pages of 128 bytes, from top to bottom; each byte describes a vertical
column of eight pixels, with bit 0 at the top and 1 meaning lit. The most recent
frame stays on screen until another frame arrives or the device restarts. Pending
frames may be replaced by newer frames if they arrive faster than the display can update.

A UDP datagram of exactly 300 bytes updates the entire strip: three bytes per LED,
in red, green, blue order, for LEDs 0 through 99. Each channel accepts 0-255.
For shorter configured strips, only the first `STRIP_LED_COUNT` triplets are used.
These packets are not forwarded to the UART outputs or interpreted as controller
or integer LED commands. When multiple RGB frames are waiting, older consecutive
frames are skipped so the strip shows the newest frame sooner; integer LED
commands retain their order relative to frames.

UDP integers from `200000000` through `299999999` update the strip using
`02AARRGGBB`: `AA` is the LED index (00-99), and `RR`, `GG`, `BB` are raw
red, green, and blue values (00-99). Indices are zero-based, so `214009900`
sets the 15th LED (index 14) to RGB (0, 99, 0), and `299990000` sets the
100th LED to RGB (99, 0, 0). Indices outside `STRIP_LED_COUNT` are logged and ignored.
Commands can be separated by whitespace, commas, or semicolons and mixed with
controller commands. LED commands also work as values in delayed IID packets:
each 16-byte entry contains a little-endian 32-bit index, 32-bit command value,
and 64-bit delay in milliseconds. The command value contains the LED index;
the IID index does not select the LED.

# How to install 

## From Window

- To Do

## From Frame

- To Do
