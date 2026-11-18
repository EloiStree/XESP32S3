
Info:
Code to download: https://github.com/EloiStree/XESP32S3
Convention for Xbox 360: https://github.com/EloiStree/S2W

Buy:  
- Buy : [ESP32-S3 Freenove WROOM-1](https://www.amazon.com.be/-/en/Freenove-Development-Dual-core-Microcontroller-Projects/dp/B0C9THDPXP/ref=sr_1_18)
- Buy : [SSD1306 I2C](https://www.amazon.com.be/s?k=ssd1306)

Install:   
- Install C++ Installer: https://visualstudio.microsoft.com/visual-cpp-build-tools/
- Install Git, optional : https://git-scm.com/
- Install Rust : Rustup.rs
  - Add Cargo to the Window System Path `%USERPROFILE%\.cargo\bin`
- Install Visual Studio: https://code.visualstudio.com/
- Install VS Rust Module : https://marketplace.visualstudio.com/items?itemName=1YiB.rust-bundle
- Install VS Python Module: https://marketplace.visualstudio.com/items?itemName=ms-python.python
- Download a python code to test if the device work : [Code](https://github.com/EloiStree/XESP32S3/blob/31c0d5646a37f8bd819504bd06dc3ce19349f197/test_code/test_code.py) 

Step:   
- Open Power Shell:
  - Check that rust did install `rustc --version` 
  - Check that cargo is install `cargo --version` 
- `cargo install espup --locked`
- `cargo install espflash --locked`
- `cargo install esp-generate --locked`
- Download the code: `git clone https://github.com/eloistree/XESP32S3.git`  
  - Go in the folder `cd XESP32S3`
    - `espup install --targets esp32s3`
- Change the COM in the `config.toml`
  -  `runner = "espflash flash --monitor --chip esp32s3 --port COM3"`
  -  Open `Window Device Manager` to check the Ports (COM)
- Check the build: `cargo run --release --bin XESP32S3`
- Create a Mobile Hotspot called `UART.LOCAL` with a password `1245789`
- Change the `WiFi` or `mDNS` in `netwrok.rs`
- Reboot the ESP32 and ping it : `ping xbox.local`
- Run the Python script to check if it works.
- Check if the ESP32 is detected as gamepad:
  - https://hardwaretester.com/gamepad
- Look at my conversion and use `UTF-8` and `\n`
  -  https://github.com/EloiStree/S2W
 
