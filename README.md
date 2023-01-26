# `jeep` CAN bus parsing for Jeep JL and 4xE

`jeep` is an easy-to-use event handling library for your Jeep. It is designed with safety in mind and is currently read-only for the Jeep's IHS network. There is also no C network support, since even connecting to it poses some risk.

**Use at your own risk. There is no warranty. Don't do dumb or illegal stuff with this library. This library is currently a WIP and the API is not yet stable. This project is not affiliated with Jeep or Stellantis.**

# Requirements

* [Rust](https://www.rust-lang.org/tools/install) toolchain.
* For development, the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension for vscode is recommended.

This library is tested to work in WSL-2 along with Raspberry Pi.

# Building

To build the library alone, for development, from this directory, run:

```bash
$ cargo build
```

To install all examples, from this directory, run:
```
$ cargo install --examples --features examples --path .
```

To view documentation in a browser offline, run:
```
cargo doc --all-features --open
```

# Examples

The [`examples`](examples) folder contains several examples, such as:
* [`jeep-alarm`](examples/alarm.rs) that runs a custom command when any doors are opened (such as a silent alarm).
* [`jeep-listen`](examples/listen.rs) that listens to the can bus and parses events in realtime.
* [`jeep-converter`](examples/converter.rs) to parse events from a `candump -L` style dump into json lines.

# Development Notes:
* This library is an in alpha state and assuredly has errors.
* See [TODO.md](TODO.md) for future plans.

# (Optional) Features
* `serde` - enables serialization of events, frames, and errors.
* `examples` - required features for [example binaries](examples).
* `embedded-can` - enables the `embedded_can::Frame` trait for our `jeep::Frame`.
* `socketcan` - enables conversion to/from `socketcan::CANFrame` and the `jeep::Listener`.

# Credits

- Code
  - Michael de Gans
- Data
  - Josh McCormick and his super useful [spreadsheet](https://docs.google.com/spreadsheets/d/16ypMADKinBBnH1pOY4-gMmVRjeR85fYplpV12aCHJC4/view)
  - [Karl Yamashita](https://github.com/karlyamashita) (for his contributions to the spreadsheet)
  - [RedRacer](https://www.jlwranglerforums.com/forum/members/redracer.10833/) for advice and testing
  - [Temperance](https://www.jlwranglerforums.com/forum/members/temperance.76687/) for finding a good source of the [TE Connectivity can bus connectors for Jeep Wrangler JL](https://www.jlwranglerforums.com/forum/threads/reverse-engineering-can-c-can-ihs-and-uds-functions.82139/page-15#post-1793144).