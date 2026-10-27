# Configuration

Set `HORIZON_VEINMINER_CONFIG` to a JSON file before starting the custom Horizon
binary. Generate a complete default file with `veinminer-config`, or edit
[`config.example.json`](../config.example.json). All fields are optional, except
a declared group's `id` and `blocks`. Unknown keys fail validation.

| Setting (`settings` object) | Default | Meaning |
|---|---:|---|
| `enabled` | true | Enable extra mining globally |
| `cooldown` | 20 | World ticks between chains, per UUID; 0..72000 |
| `mustSneak` | false | Require the acting player's sneak state |
| `maxChain` | 100 | Maximum selected blocks, including the original; 1..4096 |
| `needCorrectTool` | true | Require the game's correct tool when the block requires one for drops |
| `searchRadius` | 1 | Chebyshev neighbour radius, 1..3; 1 includes diagonals |
| `decreaseDurability` | true | Damage tools normally; extra mining stops before the final point |
| `hungerPerBlock` | 0.0 | Additional exhaustion per successful extra block; finite 0..40 |
| `separateGroupMining` | false | When true, select only the origin's block ID within its group |
| `permissionRestricted` | false | Require the player name in the top-level `players` list |

The original click always uses Horizon's original behavior and cost. Durability
protection and additional exhaustion apply only to extra blocks. When an item
has no durability limit, the durability gate does not apply. If positive
`hungerPerBlock` is configured, zero food and saturation stop extra mining.
When tool use breaks the original tool, the chain stops immediately.

```json
{
  "settings": { "mustSneak": true, "maxChain": 64 },
  "blocks": ["minecraft:nether_quartz_ore"],
  "groups": [
    {
      "id": "iron",
      "blocks": ["minecraft:iron_ore", "minecraft:deepslate_iron_ore"],
      "tools": ["minecraft:iron_pickaxe", "minecraft:diamond_pickaxe"],
      "overrides": { "maxChain": 32, "cooldown": 10 }
    }
  ],
  "players": ["Alex"]
}
```

Explicit `blocks`/`groups` arrays replace those defaults. IDs must be explicit
namespaced names, with no tag expansion or wildcard syntax. Block state
properties are ignored for group membership, so log axes and ore states can
match, but each exact state is saved in the plan and rechecked before breaking.
Unbreakable, out-of-height and unloaded blocks are excluded.

Matching groups contribute the union of their blocks and allowed tools. An
empty combined tool list permits any item that passes `needCorrectTool`.
The **first matching group in file order** supplies setting overrides. Group
IDs must be unique. `enabled` is a global switch; every other supported setting
can be overridden by a group. A standalone allowed block matches only its own ID.

Per-player session toggles apply after global/group settings and permissions.
`permissionRestricted` uses names as reported by Horizon: LAN/server guests use
their acting names, and in-frame single player uses `Player`. It is an explicit
allowlist, without a Bukkit/LuckPerms integration.

`/veinminer reload` reads the configured path again, or resets to built-in
defaults when no path was provided. Successful reload preserves session toggles
and active cooldowns; an invalid reload reports an error and retains the old
configuration. File edits have no effect until reload or restart.
