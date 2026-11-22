# Build and Flash on Steam Machine (SteamOS)

This project is firmware for an ESP32-S3, not an application that runs on the Steam Machine. Use the Steam Machine as the Linux computer that builds and flashes the firmware to a connected ESP32-S3 board.

## 1. Open a terminal

Switch to **Desktop Mode**, open Konsole, and run the commands below there. You do not need to install packages into SteamOS's system image; Rust and the Espressif tools are installed in your home directory.


Check: ❓   

## 2. Install Rust

If Rust is not installed, install it from [rustup.rs](https://rustup.rs/) and follow its Linux instructions. Open a new terminal, then check that Rust is available:

```sh
rustc --version
cargo --version
```

Check: ✅  

If Rust is not installed, these commands install it for your user:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Install or update the stable toolchain used to install Espressif's tools:

```sh
rustup toolchain install stable
```
Check: ✅ 


## 3. Install the Espressif toolchain and flasher

The project selects Espressif's `esp` toolchain in `rust-toolchain.toml`. Install the tools with stable Rust, then load the environment that `espup` creates:

```sh
These tools compile native C code. Check that the compiler in this terminal can find the standard C headers:

```sh
cc --version
printf '#include <stdint.h>\n' | cc -E -x c - >/dev/null
```

If the header check fails, do not disable SteamOS read-only mode or install packages into its system image. Use a Distrobox or another Linux development environment with a C compiler and libc development headers, and run the Rust and Espressif setup there.

Install the tools in this order:

```sh
cargo +stable install espup --locked
espup install --targets esp32s3
source "$HOME/export-esp.sh"
cargo +stable install espflash --locked
```

Check: ✅ ❓

Run `source "$HOME/export-esp.sh"` again in each new terminal before building, or add that line to your shell's startup file.

## 4. Open the project

If you already have the project, open Konsole in its folder. Otherwise, install Git for your user account if it is not already available, then clone the repository:

```sh
git clone https://github.com/eloistree/XESP32S3.git
cd XESP32S3
```
Check: ✅ ❓

## 5. Find the board's serial port

Connect the ESP32-S3 board with a USB data cable. List stable USB serial device names with:

```sh
ls -l /dev/serial/by-id/
```

Use the path shown for your board, such as `/dev/serial/by-id/usb-Your_Board-if00`. If that directory is empty or unavailable, check for `/dev/ttyACM0` or `/dev/ttyUSB0`.

Open `.cargo/config.toml` in a text editor. In the `runner` line, replace `COM25` with the device path for your board, keeping the quotes. For example:

```toml
runner = "espflash flash --chip esp32s3 --port /dev/serial/by-id/usb-Your_Board-if00"
```

Save the file. If you get a permission error when flashing, check the device's group and permissions with `ls -l /dev/ttyACM0` (or the matching device path). Your SteamOS account must have access to that serial device; log out and back in after changing any group membership.

## 6. Build and flash

From the project folder, run:

```sh
cargo run --release --bin xesp32s3
```

The first run downloads and compiles dependencies and may take several minutes. Cargo then uses `espflash` to upload the release firmware to the board. Later runs reuse the build cache.

## Troubleshooting

- If `cargo`, `espup`, or `espflash` is not found, open a new terminal. Make sure `~/.cargo/bin` is on your `PATH` and run `source "$HOME/export-esp.sh"` before building.
- If `ring` fails with `fatal error: stdint.h: No such file or directory`, run the C-header check from Step 3 in the same terminal. If it passes, retry the failed `cargo +stable install` command there; if it fails, use a Linux development container with libc headers instead of changing SteamOS system packages.
- If compilation reports a missing Espressif tool or linker, rerun `espup install --targets esp32s3`, then reload `export-esp.sh`.
- If flashing cannot find the board, confirm the USB data cable and device path, and make sure the `runner` line in `.cargo/config.toml` uses that exact path.
