## LYNX cat on rust

Currently only implements a primary.

Project is separated by workspaces. 

- Primary: Platform specific implementations for the core microcontroller
- Configs: Things like keyboard layouts, sensitivities, etc. In the future, these will be updatable by sending instructions from the device client.
- Components: Hardware-agnostic implementations for components/modules. So far, scroll-wheel and kb. next: pointers. future: LEDs, accelerometers, haptic-feedbacks, etc.


### Important TODOs

- network of microcontrollers through esp-now
- device-client driven config changes
- pointer-devices
