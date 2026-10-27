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
to a `Runtime` per game instance.

`native/horizon.rs` adds a module inside Horizon's existing game binary and
implements `Host` using its private APIs. It calls `break_block_as_player`,
retaining Horizon's loot, XP, block entities, neighbours, statistics, level
events and server-mod event behavior. It never calls `/setblock`. The existing
`with_player_actor` scope and guest-state swapping surround the chain. Extra
calls go directly to the original method, which avoids recursive chaining.
This shared entry point is reached by in-frame mining and guest mining on the
integrated/LAN/dedicated server. The adapter's survival gate excludes remote
client prediction and creative/adventure/spectator chains.

`tools/horizon.py` exports only committed files at the pinned revision using
`git archive`. It checks SHA-256 of the three adapter target files, applies
exactly-once anchors, and copies the engine/adapter/tests into the staged tree.
The original `break_block_as_player` body and Cargo manifests/lockfile are
unchanged. No Cargo dependency is added: the embedded engine uses the game's
existing serde/serde_json dependencies. Private access stays within the game
crate. Exported Horizon keeps its original license and notices.

Generated trees live under content-derived paths in `target/stages/`. A stage
is published by renaming a completed temporary directory. Reuse verifies all
recorded source hashes and keeps bytes/mtimes; edited stages fail with an
explicit error. Stage and Cargo access use Linux file locks, and Cargo artifacts
live in a separate stable target directory. No game assets or player saves are
copied. Logs/reports and exported upstream sources are ignored by Git.

The mixin engine at `../horizon-mixins` currently rejects generic impls and target
bodies containing closures/`?`. The real `Game<'a>` mining methods use all of
these. This adapter therefore uses a narrow pinned source edit, with source
hashes and original-body preservation checked independently. It is a separate
native port build. A future standard mixin bundle can replace this adapter when
those target contexts have compilation-tested support in the transformer.
