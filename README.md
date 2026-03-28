# er-extend-rs - Elden Ring Extend(e)rs

Contains various Rust crates that make use of and extend the functionality of Elden Ring bindings in [fromsoftware-rs](https://github.com/vswarte/fromsoftware-rs).

## Crates extending bindings and introducing convenience functions

- er-extend-rs-config - For loading configuration from a toml file that's alongside a dll.
- er-extend-rs-discovery - For discovering whether Elden Ring is running a known overhaul mod (currently only Reborn).
- er-extend-rs-esd - For adding options and submenus to talk scripts via structs or toml (currently only the grace menu).
- er-extend-rs-rva - For hooking into functions of known assembly footprint patterns.
- er-extend-rs-text - For overriding text values via structs or toml (currently only Event Text For Talk).

## Crates with standalone and example dlls

- standalone/absolute-weapon - Adds an option to the grace menu to upgrade all current inventory weapons to the max achieved so far.

## Credits

- [vswarte](https://github.com/vswarte), and everyone else who contributed to [fromsoftware-rs](https://github.com/vswarte/fromsoftware-rs). That project is used directly, and there are a few files included that were copied from defunct branches, and added to `er-extend-rs-esd/src/ez_state_extender/*_copy.rs` files.
- [ThomasJClark](https://github.com/ThomasJClark) for assembly footprints from [Elden Ring Glorious Merchant](https://github.com/ThomasJClark/elden-ring-glorious-merchant) and [Elden Ring Dyes](https://github.com/ThomasJClark/elden-ring-dyes)

## Licenses

- [MIT](LICENSE-MIT)
- [Apache-2.0](LICENSE-APACHE)
