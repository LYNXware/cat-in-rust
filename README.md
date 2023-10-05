## LYNX cat on rust

Project is separated by workspaces. 

- core: Receives state data from modules, applies app-settings to state, and writes keyboard reports
- Configs: Things like keyboard layouts, sensitivities, etc. In the future, these will be updatable by sending instructions from the device client.
- Components: Hardware-agnostic implementations for components/modules. So far, scroll-wheel and kb. next: pointers. future: LEDs, accelerometers, haptic-feedbacks, etc.

### Setup

flash your left cat with `cargo run --release --bin cat_esp32s3_rust_left`
flash your right cat with `cargo run --release --bin cat_esp32s3_rust_right`
flash your core cat with `cargo run --release --bin cat_esp32s3_rust_core`

With core plugged in, it will recieve events from left/right

### Important TODOs

- device-client driven config changes
- pointer-devices
- scroll wheel
- NVS storage used for peer and other app-settings
