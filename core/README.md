## Core module implementation

Currently receives esp-now messages describing key-press states, and maps them to keyboard reports.

to flash:

plug in
hold down boot button
press reset button
in root-dir, run `cargo run --release --bin cat_esp32s3_rust_core`
press reset again to start running the newly flashed firmware.

To monitor the logs, you'll need a uart-usb device, and connect via gpio pins; the usb is now used to write hid reports.
