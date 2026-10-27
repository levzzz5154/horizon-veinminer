# Horizon Veinminer

An unofficial Rust port of Veinminer's server-side connected mining behavior,
for Horizon commit `028f60881678a9a0e3e198f693a82858fa475c76` (Minecraft 26.3).
It builds a native Horizon game/server from a separate staged source tree.
The original Horizon checkout stays unchanged.

Breaking one permitted block mines its connected vein, including diagonal
neighbours. Every block goes through Horizon's existing player mining code:
loot, XP, tool wear, block updates, statistics and server block events retain
that code's behavior. Creative, adventure and spectator modes do not trigger
extra mining. The default configuration groups matching stone/deepslate ores
and matching wood species, with a maximum of 100 blocks and a 20-tick cooldown.

The engine supports explicit block/tool groups, group setting overrides,
per-player toggles, sneak/tool/permission checks, search radius, durability
protection and extra hunger cost. It plans against loaded chunks, rechecks
block states before mining, and stops when an extra break fails or the tool
is exhausted. See [configuration](docs/configuration.md).

## Build

Requirements: Linux, Rust 1.95.0, Python 3.12+, a local pinned Horizon Git clone,
and Horizon's Cargo dependencies. This checkout defaults to the sibling
`../horizon`; pass `--horizon /path/to/horizon` for another location.

```sh
cargo fetch --locked
cargo test --locked --offline
python3 tools/horizon.py check
python3 tools/horizon.py build
```

The last command prints JSON with the binary path. `build` uses Horizon's `fast`
profile and `--no-default-features`, which includes the game and dedicated server
with audio/controllers disabled. Cargo output is saved under `target/`.
Dependencies are locked and builds run offline. If Horizon's dependencies are
not cached, stage first and fetch them once:

```sh
python3 tools/horizon.py stage > target/stage.json
python3 -c 'import json; print(json.load(open("target/stage.json"))["stage"])'
# Use the printed path below:
cargo fetch --locked --manifest-path /printed/stage/path/Cargo.toml
```

Run the printed binary with your own prepared Horizon resources and game
arguments. Installing the port means using this custom binary for the world
host/server. Default single player's server thread and LAN/dedicated players
all reach the adapted player-mining entry point; a remote client does not
perform additional mining. The original Minecraft/Fabric/NeoForge/Paper client
addon protocol is separate from this port.

## Commands

- `/veinminer` or `/veinminer status`: show your session toggle.
- `/veinminer toggle`, `/veinminer on`, `/veinminer off`: change your toggle.
- `/veinminer reload`: reload the JSON configuration; permission level 2 required.

Toggles are keyed by the acting player's UUID and last for the process session.
They work without granting vanilla command privileges. Commands currently have
no custom client autocomplete and cannot be nested under `/execute`.

To customize settings:

```sh
cargo run --quiet --locked --offline --bin veinminer-config > my-veinminer.json
HORIZON_VEINMINER_CONFIG="$PWD/my-veinminer.json" /path/to/custom/horizon-client <game-arguments>
```

Without that environment variable the built-in defaults apply. A malformed
startup config disables extra mining and prints its error; ordinary mining
continues. A failed reload keeps the previously loaded configuration.

## Validation and compatibility

```sh
cargo fmt --all --check
rustfmt --edition 2024 --check native/horizon.rs
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --locked --offline
python3 tools/test_stage.py
python3 tools/horizon.py check
python3 tools/horizon.py test
```

Thirteen in-memory host tests cover connected/grouped veins, deterministic
bounded selection, exactly-once native mining calls, cooldowns/toggles,
permissions/tools/food, durability, changed blocks, failures and overflow.
`horizon.py test` embeds the same tests in the real Horizon game binary, checking
the exact engine source used there. `test_stage.py` checks repeatable bytes and
mtimes, source hashes, tamper rejection and unchanged upstream source.
A full gameplay session with prepared resources is a separate manual check.

See [architecture](docs/architecture.md), [feature coverage](docs/compatibility.md)
and [validation results](docs/validation.md). This release implements normal
connected mining; delayed mining, mining-speed adjustment, drop merging and
the client/enchantment addons remain outside its supported feature set.

[MiraculixxT's Veinminer](https://github.com/MiraculixxT/Veinminer) is the original
project. This repository contains independently written Rust code and no
upstream mod code, artwork, game binaries or Minecraft assets. License: [MIT](LICENSE).
