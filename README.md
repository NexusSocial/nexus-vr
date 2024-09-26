# NexusSocial VR Demo

This serves as a repo for the main socialvr game we are building. See the `crates`
dir for libraries and `apps` for the main binaries.

## First Time Setup

- Install [rustup](https://rustup.rs)
- Install [bevy's dependencies](https://bevyengine.org/learn/book/getting-started/setup/#install-os-dependencies)
- Install [git lfs](https://git-lfs.com/) and run `git lfs install` and `git lfs pull`
- Install `xbuild`. **It is very important to pass --git**: 
```sh
cargo install xbuild --git https://github.com/rust-mobile/xbuild
```
- Get the [Oculus SDK](https://developer.oculus.com/downloads/package/oculus-openxr-mobile-sdk/) and place `OpenXR/Libs/Android/arm64-v8a/Release/libopenxr_loader.so` into the `rumtime_libs/arm64-v8a/` folder.
- Install the [android command line tools](https://developer.android.com/tools/releases/platform-tools#downloads).
- Install an openxr openxr loader to be able to build the code natively. If you just want to cross compile to quest, this step is optional. See [here](https://monado.freedesktop.org/getting-started.html#deb) for installation of monado for linux.

## License

Unless otherwise specified, all code in this repository is dual-licensed under
either:

- MIT-0 License ([LICENSE-MIT-0](LICENSE-MIT-0))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This means you can select the license you prefer!

Any contribution intentionally submitted for inclusion in the work by you, shall be
triple licensed as above, without any additional terms or conditions.
