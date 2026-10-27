# Working on this port

Read README and docs before changing gameplay. Keep this repository separate
from Horizon and horizon-mixins. Never modify the user's upstream clone or
player data. Native edits are confined to generated, ignored staging trees.
Do not commit Minecraft assets, binaries, prepared resources or exported source.

Keep gameplay code in Rust. Test the engine through Host, preserve the original
native player-mining method, and compile the adapter against the exact pinned
Horizon revision. Configuration keys must fail closed when unsupported.
Run cargo fmt, clippy with warnings denied, cargo test, the staging test and
native check for relevant changes. Keep the feature-coverage document honest
about supported behavior and manual gameplay validation.
