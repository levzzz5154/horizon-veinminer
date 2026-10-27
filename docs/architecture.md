# Architecture

`src/lib.rs` contains a deterministic mining engine behind a small `Host`
interface. A host supplies loaded-block reads, player/tool facts and its normal
player-mining operation. The engine selects connected matching blocks by
breadth-first traversal in fixed x/y/z order. Both accepted blocks and examined
positions are bounded. Checked coordinate addition prevents wrapping at integer
limits. Preview reads do not mutate blocks or load chunks.

Selection happens before the original block is removed. The original native
mining call then runs exactly once. If it fails or removes nothing, extra mining
never starts. Each extra hit must still have its original state and pass current
tool/food checks. Changed hits are skipped; failed native breaks abort the
remaining chain. Successful extras incur only the configured additional
exhaustion. The core has no global mutable state; toggles and cooldowns belong
to a `Runtime` per world on the hosting thread.

`native/horizon.rs` adds a module inside Horizon's existing game binary and
implements `Host` using its private APIs. It calls `break_block_as_player`,
retaining Horizon's loot, XP, block entities, neighbours, statistics, level
events and server-mod event behavior. It never calls `/setblock`. The existing
`with_player_actor` scope and guest-state swapping surround the chain. Extra
calls go directly to the original method, which avoids recursive chaining.
This shared entry point is reached by in-frame mining and guest mining on the
integrated/LAN/dedicated server. The adapter's survival gate excludes remote
client prediction and creative/adventure/spectator chains.

`tools/horizon.py` exports committed Horizon files with `git archive`, checks
compatibility hashes, and materializes the committed documentation symlink.
It never replaces or appends Rust source itself. It invokes the standard
`horizon-mixins plan` and `apply` commands with `mixins/veinminer/` and records
the schema-4 JSON plan in `target/mixins/plan.json`. Three cancellable HEAD
hooks handle the mining entry, normal command dispatch and guest command
queueing. A support-module declaration installs the shared runtime. The
original native mining method and original `Game` layout/constructors stay intact.

Runtime state uses a thread-local map keyed by `world_root`. Each world has its
own UUID-keyed toggles and cooldowns. A state lease releases the RefCell borrow
before calling game code and restores state on normal return, errors or unwinding.
This avoids adding fields to `Game` and keeps server threads independent.

The self-contained runtime payload is committed at `mixins/veinminer/runtime.rs`.
`tools/bundle.py` assembles it from the adapter, core and tests; this changes
mod source only. `--check` rejects stale generated payloads. All private access
uses ordinary same-crate Rust visibility; no game-to-mod Cargo dependency is
created. Engine helpers and the module use Horizon's existing dependencies.

Mixin staging publishes a complete separate workspace atomically and preserves
unchanged bytes/mtimes. The port holds its stage lock through compilation and
locks its separate Cargo target directory. Pristine exports have source hash
markers; exported Horizon retains its license and notices. No assets or saves
are copied, and generated trees/logs are ignored by Git.

`tools/test_stage.py` applies the real bundle twice and verifies its three HEAD
injections and one module, source bytes/mtimes and immutable upstream checkout.
The Rust audit in `tools/audit` removes only planned additions and compares all
affected original files using syn and canonical formatting. This checks every
other method, struct and expression, including `break_block_as_player`, rather
than relying on a textual source replacement. Native compilation and the
embedded mining tests validate the resulting real game binary.
