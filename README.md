# NexusSocial VR Demo

A Virtual Reality Social Platform with the following goals:

- **Platform - not just framework**. We are targeting end-users not just
  developers.
- **Open Source - Free and permissively licensed** (spdx `MIT-0 OR
  Apache-2.0`).
- **Decentralized - In the Bluesky/peer-to-peer way** and not in the
  crypto/web3 way. Users should not be locked into particular server hosts.
- **Rust & Bevy** - Leveraging the best language and the best engine to punch
  above our weight.
- **Modular** - We should dogfood crates that can then be used by other people
  in the bevy ecosystem.
- **Private** - We should protect users' privacy at all costs.

If this interests you, [join the discord!][discord]

## Project Structure

* `crates/` - Mature, general-purpose crates useful for the broader ecosystem.
* `internal/` - Less mature or nexus-specific crates.
* `apps/` - The main "application entry point" crates. These are what you would
  `cargo run`.

We typically put crates under `internal` initially and only after they mature, do
we consider moving it to `crates`.

## Crate Maturity

Much of this repository is unfinished, work-in-progress code. Crate maturity is
indicated by the following:

- **WIP**: Potentially non-functional. Don't assume you can do anything but
  compile.
- **Barely Functional**: Most things work as intended, but are not stable.
  Regressions in non-critical functionality are expected. Tests might not be
  present.
- **Mature**: Code is tested, and regressions in functionality are
  *intentional* and not accidental.

No crate in this repo adheres to semver conventions, and API stability is not
followed. For this reason, none of our crates are published to crates.io.

If there is a crate here that you want to use more broadly outside of the Nexus
ecosystem, contact the maintainers and depending on our resources, we can
consider moving it to its own repo, publishing to crates.io, and maintaining
semver guarantees.

## How to Contribute

Join us in our [discord][discord]. You can also read
[CONTRIBUTING.md](./CONTRIBUTING.md) for concrete instructions on building and
how to contribute.

## License

Unless otherwise specified, all code in this repository is dual-licensed under
either:

- MIT-0 License ([LICENSE-MIT-0](LICENSE-MIT-0))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This means you can select the license you prefer!

Any contribution intentionally submitted for inclusion in the work by you, shall be
triple licensed as above, without any additional terms or conditions.

[discord]: https://discord.gg/KbdjtNaGUV
