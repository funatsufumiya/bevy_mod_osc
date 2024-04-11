# bevy_mod_osc

[![Crates.io](https://img.shields.io/crates/v/bevy_mod_osc.svg)](https://crates.io/crates/bevy_mod_osc)

OSC plugin (using [rosc](https://github.com/klingtnet/rosc)) for Bevy engine.

## Usage

- Add `bevy_mod_osc` to your `Cargo.toml`:

```toml:Cargo.toml
[dependencies]
bevy_mod_osc = "0.1"
```

- see [examples](examples)

## Version compatibility

| Bevy | bevy_osc |
|------|----------|
| 0.13 | 0.1      |

## Known issues

- on OSC Receiver, if choose `use_thread: false`, message will delay if too many messages are given at once. Recommend `use_thread: true` if you need to receive messages in real-time.
