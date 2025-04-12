# teensy4-rust-discovery
Embedded Rust with Raspberry Teensy 4

```bash
cargo generate --git https::/github.com/mciantyre/teensy4-rs-template --name teensy4-rust-discovery
cd teensy4-rust-discovery
cargo objcopy --release -- -O ihex teensy4-rust-discovery.hex
teensy_loader_cli --mcu=TEENSY40 -w -v teensy4-rust-discovery.hex
```