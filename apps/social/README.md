# Nexus Social Demo

Demos an MVP (Minimal Viable Product) for a social VR experience using the bevy
game engine.

Current goals for MVP:
- All code in Rust, with bevy game engine for client.
- Basic networking, data can be marked as synchronized. Clients have authority.
  Rollback systems are not necessary.
- Realtime Voice.
- Self-hostable server.
- Mirrors.
- Dynamically loading gltf avatar from URL
- Basic IK support
- Windows PCVR & Desktop, Linux PCVR & Desktop, MacOS Desktop, Standalone Quest.

Future Goals:
- FBT is not supported yet, but we should write the code with an exepectation that we
  will add support.
- Web is initially unsupported, due to potential incompatibility with networking.
  long term, we want to support web, at least in desktop mode.

## Project Status

This is an ambitious project, please consider it to be vaporware until proven otherwise.

## How to run

We don't have binaries published yet. For now, build the code. 
Run the [server](server/) on a computer accessbile over the network. Then run
[client](client/) either standalone on your quest or natively on your computer.

## How to Build

Be sure that you have already followed the first time setup instructions from the [toplevel README](../../README.md).

### Building for Quest Standalone

Plug in headset to PC, allow usb debugging, and then:
```sh
adb connect <device_ip> # Optional, allows wireless debugging
x devices # ensure your device is listed
x run --device <device_id_from_above> -p openxr-6dof --release
```

