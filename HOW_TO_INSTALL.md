# Build and Flash on Windows

This project is firmware for an ESP32-S3, not a Windows application. The command at the end builds the firmware and flashes it to a connected ESP32-S3 board.

## 1. Install the C++ build tools

Some Rust dependencies compile native C/C++ code on your PC. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/):

1. Download and start the Build Tools installer.
2. Select the **Desktop development with C++** workload.
3. Make sure an MSVC x64/x86 build toolset and a Windows 10 or Windows 11 SDK are selected, then install them.

## 2. Install Rust

Download and run the installer from [rustup.rs](https://rustup.rs/). Use the default installation and the `x86_64-pc-windows-msvc` host toolchain. If asked to install Visual Studio C++ build tools, complete that installation as well.

Open a new PowerShell window and check that Rust is available:

```powershell
rustc --version
cargo --version
```

## 3. Install the Espressif toolchain and flasher

Run these commands in PowerShell. `+stable` makes sure the install tools use stable Rust even though this project selects Espressif's `esp` toolchain in `rust-toolchain.toml`.

```powershell
cargo +stable install espup --locked
espup install --targets esp32s3
cargo +stable install espflash --locked
```

On Windows, `espup` sets the required toolchain environment variables automatically. Close PowerShell and open a new window after `espup install` so the updated environment is loaded.

## 4. Open the project

If you already have the project, open PowerShell in its folder. Otherwise, install [Git for Windows](https://git-scm.com/download/win), then clone the repository:

```powershell
git clone https://github.com/eloistree/XESP32S3.git
cd XESP32S3
```

## 5. Find the board's COM port

Connect the ESP32-S3 board with a USB data cable. In **Device Manager**, open **Ports (COM & LPT)** and note the COM number shown for the board, such as `COM5`.

Open `.cargo\config.toml` in a text editor. In the `runner` line, replace `COM25` with your board's COM port, keeping the quotes. For example:

```toml
runner = "espflash flash --chip esp32s3 --port COM5"
```

Save the file. If no COM port appears, try another USB data cable or install the USB-to-serial driver specified by your board manufacturer, then reconnect the board.

## 6. Build and flash

From the project folder, run:

```powershell
cargo run --release --bin xesp32s3
```

The first run downloads and compiles the dependencies and may take several minutes. Cargo then uses `espflash` to upload the release firmware to the board. Later runs reuse the build cache.

## Troubleshooting

- If PowerShell says `cargo` or `espup` is not recognized, close and reopen PowerShell. If needed, check that `%USERPROFILE%\.cargo\bin` is on your Windows `PATH`.
- If compilation reports a missing linker or C compiler, confirm that the **Desktop development with C++** workload and Windows SDK are installed, then reopen PowerShell.
- If flashing cannot find the board, confirm the COM port in Device Manager and that the `runner` line in `.cargo\config.toml` uses that exact port.