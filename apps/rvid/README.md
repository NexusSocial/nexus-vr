# RViD - Remote Virtual Display

A rust based wireless PCVR solution. We intend to support:
- Streaming your desktop to your Quest
- Playing PC Virtual Reality games wirelessly.
- Windows, Linux, *and* MacOS.
- Implementing (the first?) an openxr runtime for MacOS.
- Interacting with desktop via hand tracking, including controller emulation.
- Keeping the dependencies 100% Rust whenever possible.

Non-goals (for now):
- Headsets other than Quest (but we will design the code to be portable).

## Project Status

This is an ambitious project, please consider it to be vaporware until proven otherwise.

## How to run

We don't have binaries published yet. For now, build the code. 
Run the [server](server/) on your computer and [client](client/) on your headset.

## How to Build

Be sure that you have already followed the first time setup instructions from the [toplevel README](../../README.md).

Plug in headset to PC, allow usb debugging, and then:
```sh
adb connect <device_ip> # Optional, allows wireless debugging
x devices # ensure your device is listed
x run --device <device_id_from_above> -p openxr-6dof --release
```

