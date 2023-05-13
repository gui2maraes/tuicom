# TuiCOM

A `tui` based serial terminal written in Rust.

## Features
- Baud rate and port selection
- Separate TX and RX consoles
- View TX and RX as hex
- More to come

## Bindings
- `q`: Quit
- `H`: Switch hex TX output
- `h`: Switch hex RX output
- `C`: Clear TX
- `c`: Clear RX
- `l`: Switch LF to CR + LF
- `b`: Change baud rate
- `i`: Enter INSERT mode
- `Esc`: Enter NORMAL mode

## Building

Just run `cargo install --path .`.

### Dependencies

For GNU/Linux pkg-config headers are required:

- Ubuntu: `sudo apt install pkg-config`
- Fedora: `sudo dnf install pkgconf-pkg-config`

For other distros they may provide pkg-config through the pkgconf package instead.

For GNU/Linux libudev headers are required as well:

- Ubuntu: `sudo apt install libudev-dev`
- Fedora: `sudo dnf install systemd-devel`
