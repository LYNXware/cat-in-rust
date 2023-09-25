## LYNX cat on rust

Currently only implements a primary.

Project is separated by workspaces. 

- core: Platform specific implementations for the core microcontroller
- Configs: Things like keyboard layouts, sensitivities, etc. In the future, these will be updatable by sending instructions from the device client.
- Components: Hardware-agnostic implementations for components/modules. So far, scroll-wheel and kb. next: pointers. future: LEDs, accelerometers, haptic-feedbacks, etc.

### Setup

flash your left cat with `cargo run --release --bin cat_esp32s3_rust_left`
flash your right cat with `cargo run --release --bin cat_esp32s3_rust_right`

power on your left and right cats (just power)

see their states sent to core:

flash and monitor your core module with  `cargo run --release --bin cat_esp32s3_rust_core`

Press the buttons. you only see bit-packed bytes of the matrix state for now. Once data-processing is implemented, they will be keyboard events.

### Important TODOs

- interprate states into hid reports
- device-client driven config changes
- pointer-devices
