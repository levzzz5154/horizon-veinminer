# Validation results

Validated on Linux x86_64, 2026-10-28, Rust 1.95.0, against Horizon
`028f60881678a9a0e3e198f693a82858fa475c76`.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Passed |
| `rustfmt --edition 2024 --check native/horizon.rs` | Passed |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo test --locked --offline` | 13 engine tests passed; config utility compiles |
| `python3 tools/test_stage.py` | Three real mixin HEADs and one module verified; repeated bytes/mtimes preserved; canonical AST audit and unchanged checkout |
| `python3 tools/horizon.py check` | Real Horizon game binary with native adapter type-checks, no default audio/controller features |
| `python3 tools/horizon.py test` | Same 13 engine tests passed inside the linked Horizon game test binary; 135 unrelated upstream tests filtered |

Before migration to the TOML bundle, the initial native test build took 56.430 seconds, including dependency
compilation and linking. A cached repeated native check on the earlier adapter
revision took 0.162 seconds; rebuilding after an adapter/source-path change took
17.682 seconds. These are local development measurements, not full optimized game
build or startup estimates. Logs live in `target/horizon-{check,test}.log`.

The unit tests use an in-memory `Host`. They verify forwarding to the native
mining operation once per selected block and engine behavior; they do not test
Horizon loot-table contents or network delivery. Native compilation verifies
integration with the real private game methods and state types. The staged
source test audits every affected original file after removing only planned
HEADs and the runtime module; the original mining body and Game layout are unchanged.

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

The mixin-based 0.2.0 port passed the same 13 embedded tests in the real game
binary. Its first native check took 25.882 s and native test build took 49.394 s.
Planning/applying the complete 3382-file source export took about 26 s, including
two full planner passes; this cost remains even when no source changes.
The earlier fast cached compilation number is not a full-workspace planning
measurement. Engine 0.4.0 independently passes 48 integration tests, including
positive/negative lifetime HEAD and support-module cases plus snapshots.
