[<img width="764" height="939" alt="image" src="https://github.com/user-attachments/assets/b401b0f5-637f-48fe-84ce-6eceddfa6570" />](https://github.com/user-attachments/assets/87e4596e-861f-4d38-b943-54d9d2fccf19)


The aim of this toolbox is to simulate an XInput Xbox 360 controller, allowing you to play games while learning how to code.   
Learning to code involves understanding concepts such as 2D arrays and images. I recommend buying an SSD1306 display to practise your coding skills.   
Since coding is also closely linked to colours, you can buy a 100-LED strip to experiment with RGB lighting.   
If you buy an ESP32, you might also want to have some fun building a small four-wheeled car using the L298N motor controller, which can also be used as a power source.     

# Hardware  
  
**Freenove Board**       
[<img width="1055" height="761" alt="image" src="https://github.com/user-attachments/assets/104f7282-4b8d-4279-81d3-96097f4b6069" />](https://www.amazon.com.be/Freenove-Universal-Breakout-Raspberry-Switchable/dp/B0DT48T7CB)   
https://www.amazon.com.be/Freenove-Universal-Breakout-Raspberry-Switchable/dp/B0DT48T7CB    

**ESP32-S3**    
[<img width="1002" height="803" alt="image" src="https://github.com/user-attachments/assets/9ca93073-cba8-4606-acb3-10cbdd6e80ac" />](https://www.amazon.com.be/-/en/Freenove-ESP32-S3-Development-Dual-core-Microcontroller/dp/B0DHJZ1V81/)  
https://www.amazon.com.be/-/en/Freenove-ESP32-S3-Development-Dual-core-Microcontroller/dp/B0DHJZ1V81/    

  

**SSD1306 Display**     
[<img width="631" height="339" alt="image" src="https://github.com/user-attachments/assets/7789ec0a-0545-4f69-8058-b58b31eccd7f" />](https://www.amazon.com.be/s?k=ssd1306)    
https://www.amazon.com.be/s?k=ssd1306   

   

**5V WS2812B RGB LED Strip (1 m)**    
[<img width="864" height="425" alt="image" src="https://github.com/user-attachments/assets/08be6ae1-1eb3-4b9e-9ec4-f23696659f17" />](https://www.amazon.com.be/dp/B0D5B7HDPP)   
https://www.amazon.com.be/dp/B0D5B7HDPP  

   

**USB Power Supply for the LED Strip**   
You will need a USB power supply to power the LED strip connected to the Freenove board.  
[<img width="839" height="560" alt="image" src="https://github.com/user-attachments/assets/fcb6c01a-cba8-49c8-890b-29b5daff7766" />](https://www.amazon.com.be/dp/B0DNYGKXH5)   
https://www.amazon.com.be/dp/B0DNYGKXH5   

   

**Level Shifter**    
As you will most probably want to use microcontrollers that don't have Wi-Fi, a level shifter will help you connect and use them from a Godot/Unity application.      
[<img width="792" height="398" alt="image" src="https://github.com/user-attachments/assets/e8c752a6-6612-4562-bc7a-d6f4bf321b55" />](https://www.amazon.com.be/s?k=level+shifter)   
https://www.amazon.com.be/s?k=level+shifter    

   
 

**Paper Ink**
[<img width="1192" height="775" alt="image" src="https://github.com/user-attachments/assets/bfd40cf5-9b90-4b78-ac86-82d993102f1d" />](https://www.amazon.com.be/s?k=200x200+eink+sharewave)
https://www.amazon.com.be/s?k=200x200+eink+sharewave




# Optional: Build a Small Car

If you want to take things a step further, you can build a small car using the following components:

**L298N Motor Controller Module**    
[<img width="690" height="655" alt="image" src="https://github.com/user-attachments/assets/cc863862-c948-46ab-b9ac-b6f7ea006374" />](https://www.amazon.com.be/-/en/Controller-Stepper-Capacity-Regulator-Projects/dp/B0GXF8HB48)   
https://www.amazon.com.be/-/en/Controller-Stepper-Capacity-Regulator-Projects/dp/B0GXF8HB48   


**Motors**     
[<img width="409" height="331" alt="image" src="https://github.com/user-attachments/assets/c9f16f23-d17d-4cb4-b286-b9cd4de79b6e" />](https://www.amazon.com.be/-/en/3V-6V-Shaft-200RPM-intelligent-double/dp/B0FB3FLZ26/)   
https://www.amazon.com.be/-/en/3V-6V-Shaft-200RPM-intelligent-double/dp/B0FB3FLZ26/   



# Optional: Add a Camera

Make your project even more fun by adding a camera.

**XIAO ESP32-S3 Sense**    
[<img width="1078" height="795" alt="image" src="https://github.com/user-attachments/assets/33b04e9a-e3b6-4f59-9f15-965e1ac91a1d" />](https://www.amazon.com.be/-/en/dp/B0C69FFVHH)   
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

Put the screen on the 3v3 pin of the Freenova.
