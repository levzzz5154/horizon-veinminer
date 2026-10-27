# Feature coverage

Reference project: [MiraculixxT/Veinminer](https://github.com/MiraculixxT/Veinminer),
revision `b3bfeb39945260f857a9b79c11dc206c109a9d0c`.
Its [overview](https://github.com/MiraculixxT/Veinminer/blob/b3bfeb39945260f857a9b79c11dc206c109a9d0c/docs/overview.md)
describes the base server mod and separate client/enchantment addons. This is an
unofficial behavioral port of normal server-side vein mining, with an independent
configuration format and native Horizon adapter.

| Capability | This version |
|---|---|
| Connected veins including diagonal neighbours | Implemented; fixed-order bounded breadth-first traversal |
| Normal loot, XP, tool wear, block updates and events | Delegates every block to the existing Horizon player-mining method |
| Stone/deepslate ore and wood groups | Implemented with explicit IDs and independently chosen defaults |
| Custom blocks, tool groups and group settings | Implemented in JSON; first matching group supplies overrides |
| Correct-tool, survival, sneak and durability checks | Implemented |
| Cooldown, per-player toggles, extra hunger cost | Implemented |
| Permission restriction | Explicit name allowlist; no permissions plugin integration |
| Settings reload | Implemented with permission level 2 and error retention |
| Single player, LAN and dedicated server path | Shared native adapter type-checked; full live sessions require manual verification |
| Client addon keybind, previews, HUD, tunnel/flat shapes | Future work; the core exposes a pure preview API |
| Enchantment addon | Future work |
| Delayed/layered breaking and mining-speed adjustment | Future work; current chains run synchronously after the original break |
| Merged drop positions | Future work; native drops remain at their own blocks |
| Upstream `/settings`, `/groups`, `/presets`, `/blocks` editing | Future work; edit JSON and reload |
| Tags, wildcard IDs and client/server configuration packets | Future work; explicit IDs and server configuration only |
| Automatic updates | Future work |
| Native mixin bundle | Implemented: three TOML HEAD injections and one support module, applied by horizon-mixins 0.4.0 |
| `.hmod` packaging | Future work; the bundle builds a pinned custom native binary |

The maximum chain counts eligible selected blocks, including the original.
Search radius is Chebyshev distance around each accepted block; larger radii
can bridge gaps. This is a documented source-level contract, rather than a
promise of identical traversal ordering to upstream. Runtime changes use
Horizon's existing vanilla mining behavior and random call order for each
individual block. Mods that cancel the underlying native operation are
respected when it reports no removal. WASM `block-broken` callbacks are
after-the-fact notifications and cannot cancel mining.

No Minecraft assets, upstream mod source or artwork are distributed here.
Preparing resources, signing into Minecraft and running Horizon remain the
host application's responsibility. Generated exports preserve Horizon's own
license and third-party notices.
