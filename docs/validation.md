# Validation results

Validated on Linux x86_64, 2026-10-28, Rust 1.95.0, against Horizon
`028f60881678a9a0e3e198f693a82858fa475c76`.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Passed |
| `rustfmt --edition 2024 --check native/horizon.rs` | Passed |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo test --locked --offline` | 13 engine tests passed; config utility compiles |
| `python3 tools/test_stage.py` | 3385 exported/added files verified; repeated bytes/mtimes preserved; edited stage rejected; original mining body and checkout unchanged |
| `python3 tools/horizon.py check` | Real Horizon game binary with native adapter type-checks, no default audio/controller features |
| `python3 tools/horizon.py test` | Same 13 engine tests passed inside the linked Horizon game test binary; 135 unrelated upstream tests filtered |

The initial native test build took 56.430 seconds, including dependency
compilation and linking. A cached repeated native check on the earlier adapter
revision took 0.162 seconds; rebuilding after an adapter/source-path change took
17.682 seconds. These are local development measurements, not full optimized game
build or startup estimates. Logs live in `target/horizon-{check,test}.log`.

The unit tests use an in-memory `Host`. They verify forwarding to the native
mining operation once per selected block and engine behavior; they do not test
Horizon loot-table contents or network delivery. Native compilation verifies
integration with the real private game methods and state types. The staged
source test independently proves the existing player-mining body is unchanged.

No game process was launched and no prepared Minecraft resources or real player
saves were used. Before distributing a playable binary, run manual checks in
fresh isolated `HORIZON_HOME`, `XDG_DATA_HOME` and `XDG_CONFIG_HOME` folders:

1. In survival, mine a mixed iron/deepslate-iron vein. Confirm all matching loaded
   blocks break, drops/XP arrive, the tool wears per block and a distant vein stays.
2. Repeat with sneak/tool restrictions, low durability, hunger cost, and cooldown.
3. Test `/veinminer off/on/status` for two players; confirm toggles and cooldowns
   stay separate. Check reload denial for a non-operator and success at level 2.
4. Repeat in default server-thread single player, in-frame mode and a dedicated
   or LAN host with a second client. Confirm each client sees the same changes.
5. Verify creative, adventure and spectator actions never trigger a chain.

Set `HORIZON_NO_AUDIO=1` and use scratch launcher directories too if a launcher
is involved. Remove only the scratch data after the test. These live sessions
remain pending; the completed validation is compilation, engine behavior and
staging integrity.
