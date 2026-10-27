// Original Rust implementation of the documented vein-mining behavior.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub type Position = [i32; 3];
pub type Actor = [i32; 4];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Settings {
    pub enabled: bool,
    pub cooldown: i64,
    pub must_sneak: bool,
    pub max_chain: usize,
    pub need_correct_tool: bool,
    pub search_radius: i32,
    pub decrease_durability: bool,
    pub hunger_per_block: f32,
    pub separate_group_mining: bool,
    pub permission_restricted: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown: 20,
            must_sneak: false,
            max_chain: 100,
            need_correct_tool: true,
            search_radius: 1,
            decrease_durability: true,
            hunger_per_block: 0.0,
            separate_group_mining: false,
            permission_restricted: false,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=4096).contains(&self.max_chain) {
            return Err("maxChain must be 1..4096 (including the original block)".into());
        }
        if !(1..=3).contains(&self.search_radius) {
            return Err("searchRadius must be 1..3".into());
        }
        if !(0..=72_000).contains(&self.cooldown) {
            return Err("cooldown must be 0..72000 ticks".into());
        }
        if !self.hunger_per_block.is_finite() || !(0.0..=40.0).contains(&self.hunger_per_block) {
            return Err("hungerPerBlock must be finite and 0..40".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Overrides {
    pub cooldown: Option<i64>,
    pub must_sneak: Option<bool>,
    pub max_chain: Option<usize>,
    pub need_correct_tool: Option<bool>,
    pub search_radius: Option<i32>,
    pub decrease_durability: Option<bool>,
    pub hunger_per_block: Option<f32>,
    pub separate_group_mining: Option<bool>,
    pub permission_restricted: Option<bool>,
}
impl Overrides {
    fn apply(&self, settings: &mut Settings) {
        macro_rules! fields { ($($field:ident),*) => { $(if let Some(value) = self.$field { settings.$field = value; })* }; }
        fields!(
            cooldown,
            must_sneak,
            max_chain,
            need_correct_tool,
            search_radius,
            decrease_durability,
            hunger_per_block,
            separate_group_mining,
            permission_restricted
        );
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub blocks: BTreeSet<String>,
    #[serde(default)]
    pub tools: BTreeSet<String>,
    #[serde(default)]
    pub overrides: Overrides,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub settings: Settings,
    pub blocks: BTreeSet<String>,
    pub groups: Vec<Group>,
    pub players: BTreeSet<String>,
}
fn id_valid(s: &str) -> bool {
    let Some((namespace, path)) = s.split_once(':') else {
        return false;
    };
    !namespace.is_empty()
        && !path.is_empty()
        && namespace
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-".contains(&b))
        && path
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"/_.-".contains(&b))
}
impl Config {
    pub fn from_json(source: &str) -> Result<Self, String> {
        let config: Self = serde_json::from_str(source).map_err(|e| e.to_string())?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.settings.validate()?;
        if self.groups.len() > 256 || self.blocks.len() > 4096 {
            return Err("too many groups or blocks".into());
        }
        let mut ids = BTreeSet::new();
        for group in &self.groups {
            if group.id.is_empty() || !ids.insert(&group.id) {
                return Err(format!("empty/duplicate group id: {}", group.id));
            }
            if group.blocks.is_empty() || group.blocks.len() > 4096 || group.tools.len() > 4096 {
                return Err(format!("group {} has invalid block/tool count", group.id));
            }
            let mut settings = self.settings.clone();
            group.overrides.apply(&mut settings);
            settings
                .validate()
                .map_err(|e| format!("group {}: {e}", group.id))?;
        }
        for id in self.blocks.iter().chain(
            self.groups
                .iter()
                .flat_map(|g| g.blocks.iter().chain(&g.tools)),
        ) {
            if !id_valid(id) {
                return Err(format!(
                    "expected explicit namespaced block/item id, got {id:?}"
                ));
            }
        }
        Ok(())
    }
    pub fn action(&self, name: &str, tool: Option<&str>) -> Option<(Settings, BTreeSet<String>)> {
        let groups: Vec<_> = self
            .groups
            .iter()
            .filter(|g| g.blocks.contains(name))
            .collect();
        if groups.is_empty() && !self.blocks.contains(name) {
            return None;
        }
        let mut settings = self.settings.clone();
        if let Some(first) = groups.first() {
            first.overrides.apply(&mut settings);
        }
        let tools: BTreeSet<_> = groups.iter().flat_map(|g| &g.tools).collect();
        if !tools.is_empty()
            && tool.is_none_or(|id| !tools.iter().any(|allowed| allowed.as_str() == id))
        {
            return None;
        }
        let targets = if settings.separate_group_mining || groups.is_empty() {
            BTreeSet::from([name.to_owned()])
        } else {
            groups
                .iter()
                .flat_map(|g| g.blocks.iter().cloned())
                .collect()
        };
        Some((settings, targets))
    }
}
impl Default for Config {
    fn default() -> Self {
        let mut groups = Vec::new();
        for ore in [
            "coal", "copper", "iron", "gold", "redstone", "lapis", "diamond", "emerald",
        ] {
            groups.push(Group {
                id: format!("{ore}-ore"),
                blocks: [
                    format!("minecraft:{ore}_ore"),
                    format!("minecraft:deepslate_{ore}_ore"),
                ]
                .into(),
                tools: BTreeSet::new(),
                overrides: Overrides::default(),
            });
        }
        for wood in [
            "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove", "cherry",
            "pale_oak",
        ] {
            groups.push(Group {
                id: format!("{wood}-logs"),
                blocks: [
                    format!("minecraft:{wood}_log"),
                    format!("minecraft:{wood}_wood"),
                ]
                .into(),
                tools: BTreeSet::new(),
                overrides: Overrides::default(),
            });
        }
        Self {
            settings: Settings::default(),
            blocks: [
                "minecraft:nether_quartz_ore".into(),
                "minecraft:nether_gold_ore".into(),
                "minecraft:ancient_debris".into(),
            ]
            .into(),
            groups,
            players: BTreeSet::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub state: u32,
    pub breakable: bool,
}
#[derive(Clone, Debug)]
pub struct Player {
    pub actor: Actor,
    pub name: String,
    pub tick: i64,
    pub survival: bool,
    pub sneaking: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tool {
    pub item: String,
    pub remaining: Option<u32>,
}

pub trait Host {
    type Error;
    fn player(&mut self) -> Player;
    // None means unloaded/outside build bounds. This must never load a chunk.
    fn block(&self, position: Position) -> Option<Block>;
    fn tool(&self) -> Option<Tool>;
    fn can_harvest(&self, block: &Block) -> bool;
    fn has_food(&self) -> bool;
    // Must use the game's ordinary player-mining path, never /setblock.
    fn mine(&mut self, position: Position, preserve_durability: bool) -> Result<bool, Self::Error>;
    fn exhaust(&mut self, amount: f32);
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub position: Position,
    pub block: Block,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub planned: usize,
    pub extra_broken: usize,
    pub changed: usize,
    pub stopped: bool,
}

pub fn preview<H: Host>(
    host: &H,
    origin: Position,
    targets: &BTreeSet<String>,
    settings: &Settings,
) -> Vec<Hit> {
    let mut hits = Vec::new();
    if settings.validate().is_err() {
        return hits;
    }
    let mut seen = BTreeSet::from([origin]);
    let mut queue = VecDeque::from([origin]);
    // This bound caps reads of nonmatching neighbours as well as accepted blocks.
    let max_examined = settings.max_chain.saturating_mul(343).min(1_000_000);
    while let Some(position) = queue.pop_front() {
        if hits.len() >= settings.max_chain {
            break;
        }
        let Some(block) = host.block(position) else {
            continue;
        };
        if !block.breakable
            || !targets.contains(&block.name)
            || settings.need_correct_tool && !host.can_harvest(&block)
        {
            continue;
        }
        hits.push(Hit { position, block });
        let r = settings.search_radius;
        for x in -r..=r {
            for y in -r..=r {
                for z in -r..=r {
                    let (Some(x), Some(y), Some(z)) = (
                        position[0].checked_add(x),
                        position[1].checked_add(y),
                        position[2].checked_add(z),
                    ) else {
                        continue;
                    };
                    let next = [x, y, z];
                    if seen.len() < max_examined && seen.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
        }
    }
    hits
}

#[derive(Default)]
pub struct Runtime {
    cooldowns: BTreeMap<Actor, i64>,
    disabled: BTreeSet<Actor>,
}
impl Runtime {
    pub fn enabled(&self, actor: Actor) -> bool {
        !self.disabled.contains(&actor)
    }
    pub fn set_enabled(&mut self, actor: Actor, enabled: bool) {
        if enabled {
            self.disabled.remove(&actor);
        } else {
            self.disabled.insert(actor);
        }
    }
    pub fn mine<H: Host>(
        &mut self,
        host: &mut H,
        config: &Config,
        origin: Position,
    ) -> Result<Report, H::Error> {
        let player = host.player();
        let tool = host.tool();
        let seed = host.block(origin);
        let action = seed
            .as_ref()
            .filter(|b| b.breakable)
            .and_then(|b| config.action(&b.name, tool.as_ref().map(|t| t.item.as_str())));
        let action = action.filter(|(settings, _)| {
            settings.enabled
                && player.survival
                && self.enabled(player.actor)
                && (!settings.must_sneak || player.sneaking)
                && (!settings.permission_restricted || config.players.contains(&player.name))
                && self
                    .cooldowns
                    .get(&player.actor)
                    .is_none_or(|end| player.tick >= *end)
        });
        let hits = if let Some((settings, targets)) = &action {
            preview(host, origin, targets, settings)
        } else {
            Vec::new()
        };
        let mut report = Report {
            planned: hits.len(),
            ..Report::default()
        };
        // The original click always follows Horizon's original behavior, even if the mod is off.
        if !host.mine(origin, false)? || hits.len() <= 1 {
            return Ok(report);
        }
        let Some((settings, _)) = action else {
            return Ok(report);
        };
        self.cooldowns.retain(|_, end| *end > player.tick);
        self.cooldowns
            .insert(player.actor, player.tick.saturating_add(settings.cooldown));
        for hit in hits.into_iter().skip(1) {
            if host.tool() != tool
                && host.tool().as_ref().map(|t| &t.item) != tool.as_ref().map(|t| &t.item)
            {
                report.stopped = true;
                break;
            }
            if settings.decrease_durability
                && host
                    .tool()
                    .as_ref()
                    .and_then(|t| t.remaining)
                    .is_some_and(|n| n <= 1)
                || settings.hunger_per_block > 0.0 && !host.has_food()
            {
                report.stopped = true;
                break;
            }
            if host.block(hit.position).as_ref() != Some(&hit.block) {
                report.changed += 1;
                continue;
            }
            if settings.need_correct_tool && !host.can_harvest(&hit.block) {
                report.stopped = true;
                break;
            }
            if !host.mine(hit.position, !settings.decrease_durability)? {
                report.stopped = true;
                break;
            }
            host.exhaust(settings.hunger_per_block);
            report.extra_broken += 1;
        }
        Ok(report)
    }
}
