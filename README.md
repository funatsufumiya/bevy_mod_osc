# bevy_mod_osc

[![Crates.io](https://img.shields.io/crates/v/bevy_mod_osc.svg)](https://crates.io/crates/bevy_mod_osc)
[![Docs.rs](https://docs.rs/bevy_mod_osc/badge.svg)](https://docs.rs/bevy_mod_osc)
[![License](https://img.shields.io/crates/l/bevy_mod_osc.svg)](LICENSE)

OSC plugin (using [rosc](https://github.com/klingtnet/rosc)) for Bevy engine.

## Features

- You can choose IPv4 or IPv6
- You can choose using thread or not (recommend to use thread. see [Known Issues](#known-issues))

## Usage

- Add `bevy_mod_osc` to your `Cargo.toml`:

```toml:Cargo.toml
[dependencies]
bevy_mod_osc = "0.2"
```

- see [examples](examples)

## Version compatibility

| Bevy | bevy_osc |
|------|----------|
| 0.14 | 0.2      |
| 0.13 | 0.1      |

## Known issues

- on OSC Receiver, if choose `use_thread: false`, message will delay if too many messages are given at once. Recommend `use_thread: true` if you need to receive messages in real-time.
