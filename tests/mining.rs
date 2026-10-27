use horizon_veinminer::*;
use std::collections::{BTreeMap, BTreeSet};

struct World {
    blocks: BTreeMap<Position, Block>,
    player: Player,
    tool: Option<Tool>,
    food: bool,
    harvest: bool,
    calls: Vec<(Position, bool)>,
    drops: usize,
    xp: usize,
    exhaustion: f32,
    failed: BTreeSet<Position>,
    error: Option<Position>,
    change_after_origin: Option<Position>,
}
impl World {
    fn new(points: &[Position]) -> Self {
        Self {
            blocks: points
                .iter()
                .map(|p| (*p, block("minecraft:iron_ore", 1)))
                .collect(),
            player: Player {
                actor: [0, 0, 0, 1],
                name: "Alex".into(),
                tick: 0,
                survival: true,
                sneaking: false,
            },
            tool: Some(Tool {
                item: "minecraft:iron_pickaxe".into(),
                remaining: Some(1000),
            }),
            food: true,
            harvest: true,
            calls: Vec::new(),
            drops: 0,
            xp: 0,
            exhaustion: 0.0,
            failed: BTreeSet::new(),
            error: None,
            change_after_origin: None,
        }
    }
}
fn block(name: &str, state: u32) -> Block {
    Block {
        name: name.into(),
        state,
        breakable: true,
    }
}
impl Host for World {
    type Error = &'static str;
    fn player(&mut self) -> Player {
        self.player.clone()
    }
    fn block(&self, p: Position) -> Option<Block> {
        self.blocks.get(&p).cloned()
    }
    fn tool(&self) -> Option<Tool> {
        self.tool.clone()
    }
    fn can_harvest(&self, _: &Block) -> bool {
        self.harvest
    }
    fn has_food(&self) -> bool {
        self.food
    }
    fn mine(&mut self, p: Position, preserve: bool) -> Result<bool, Self::Error> {
        self.calls.push((p, preserve));
        if self.error == Some(p) {
            return Err("native mining failed");
        }
        if self.failed.contains(&p) {
            return Ok(false);
        }
        let removed = self.blocks.remove(&p).is_some();
        if removed {
            self.drops += 1;
            self.xp += 2;
            if !preserve
                && let Some(t) = self.tool.as_mut()
                && let Some(n) = t.remaining.as_mut()
            {
                *n = n.saturating_sub(1);
                if *n == 0 {
                    self.tool = None;
                }
            }
        }
        if self.calls.len() == 1
            && let Some(p) = self.change_after_origin
        {
            self.blocks.insert(p, block("minecraft:diamond_ore", 99));
        }
        Ok(removed)
    }
    fn exhaust(&mut self, n: f32) {
        self.exhaustion += n;
    }
}
fn mine(world: &mut World, config: &Config) -> Report {
    Runtime::default().mine(world, config, [0, 0, 0]).unwrap()
}

#[test]
fn connected_diagonals_group_variants_and_original_once_keep_drops_and_xp() {
    let mut w = World::new(&[[0, 0, 0], [1, 1, 1], [2, 1, 1], [20, 0, 0]]);
    w.blocks
        .insert([2, 1, 1], block("minecraft:deepslate_iron_ore", 2));
    let report = mine(&mut w, &Config::default());
    assert_eq!((report.planned, report.extra_broken), (3, 2));
    assert_eq!((w.calls.len(), w.drops, w.xp), (3, 3, 6));
    assert_eq!(w.calls.iter().filter(|(p, _)| *p == [0, 0, 0]).count(), 1);
    assert!(w.blocks.contains_key(&[20, 0, 0]));
    assert_eq!(w.tool.unwrap().remaining, Some(997));
}
#[test]
fn plan_is_pure_deterministic_bounded_and_max_chain_includes_origin() {
    let points: Vec<_> = (-8..=8)
        .flat_map(|x| (-8..=8).map(move |z| [x, 0, z]))
        .collect();
    let mut w = World::new(&points);
    let mut c = Config::default();
    c.settings.max_chain = 7;
    let targets = BTreeSet::from(["minecraft:iron_ore".into()]);
    let a = preview(&w, [0, 0, 0], &targets, &c.settings);
    let b = preview(&w, [0, 0, 0], &targets, &c.settings);
    assert_eq!(a, b);
    assert_eq!(a.len(), 7);
    assert_eq!(w.calls.len(), 0);
    assert_eq!(mine(&mut w, &c).extra_broken, 6);
    assert_eq!(w.calls.len(), 7);
}
#[test]
fn loaded_matching_blocks_only_no_generated_chunks_or_unbreakable_bridge() {
    let mut w = World::new(&[[0, 0, 0], [2, 0, 0]]);
    w.blocks.insert([1, 0, 0], block("minecraft:stone", 2));
    assert_eq!(mine(&mut w, &Config::default()).extra_broken, 0);
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    w.blocks.get_mut(&[1, 0, 0]).unwrap().breakable = false;
    assert_eq!(mine(&mut w, &Config::default()).extra_broken, 0);
    let mut w = World::new(&[[0, 0, 0], [2, 0, 0]]);
    let mut c = Config::default();
    c.settings.search_radius = 2;
    assert_eq!(mine(&mut w, &c).extra_broken, 1);
}
#[test]
fn last_durability_point_is_kept_and_broken_initial_tool_stops_chain() {
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0], [3, 0, 0]]);
    w.tool.as_mut().unwrap().remaining = Some(3);
    let r = mine(&mut w, &Config::default());
    assert_eq!(r.extra_broken, 1);
    assert!(r.stopped);
    assert_eq!(w.tool.unwrap().remaining, Some(1));
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    w.tool.as_mut().unwrap().remaining = Some(1);
    assert_eq!(mine(&mut w, &Config::default()).extra_broken, 0);
    assert!(w.tool.is_none());
}
#[test]
fn preservation_and_additional_exhaustion_only_apply_to_extra_blocks() {
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    w.tool.as_mut().unwrap().remaining = Some(2);
    let mut c = Config::default();
    c.settings.decrease_durability = false;
    c.settings.hunger_per_block = 0.5;
    assert_eq!(mine(&mut w, &c).extra_broken, 2);
    assert_eq!(
        w.calls,
        vec![([0, 0, 0], false), ([1, 0, 0], true), ([2, 0, 0], true)]
    );
    assert_eq!(w.tool.unwrap().remaining, Some(1));
    assert_eq!(w.exhaustion, 1.0);
}
#[test]
fn stale_blocks_are_not_mined_even_when_the_new_block_is_whitelisted() {
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    w.change_after_origin = Some([1, 0, 0]);
    let r = mine(&mut w, &Config::default());
    assert_eq!((r.changed, r.extra_broken), (1, 1));
    assert_eq!(w.blocks[&[1, 0, 0]].name, "minecraft:diamond_ore");
}
#[test]
fn original_failure_never_triggers_extras_and_extra_failure_aborts() {
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    w.failed.insert([0, 0, 0]);
    assert_eq!(mine(&mut w, &Config::default()).extra_broken, 0);
    assert_eq!(w.calls.len(), 1);
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    w.failed.insert([1, 0, 0]);
    let r = mine(&mut w, &Config::default());
    assert!(r.stopped);
    assert_eq!(w.calls.len(), 2);
}
#[test]
fn native_errors_propagate_without_attempting_later_blocks() {
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    w.error = Some([1, 0, 0]);
    assert_eq!(
        Runtime::default().mine(&mut w, &Config::default(), [0, 0, 0]),
        Err("native mining failed")
    );
    assert_eq!(w.calls.len(), 2);
}
#[test]
fn creative_adventure_sneak_tool_food_permissions_and_global_disable_gate_only_extras() {
    for gate in 0..7 {
        let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
        let mut c = Config::default();
        match gate {
            0 => w.player.survival = false,
            1 => c.settings.must_sneak = true,
            2 => w.harvest = false,
            3 => {
                c.settings.hunger_per_block = 0.2;
                w.food = false;
            }
            4 => c.settings.permission_restricted = true,
            5 => c.settings.enabled = false,
            _ => {
                c.groups.clear();
                c.blocks.clear();
            }
        }
        assert_eq!(mine(&mut w, &c).extra_broken, 0, "gate {gate}");
        assert_eq!(w.calls.len(), 1);
    }
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    w.player.sneaking = true;
    let mut c = Config::default();
    c.settings.must_sneak = true;
    c.settings.permission_restricted = true;
    c.players.insert("Alex".into());
    assert_eq!(mine(&mut w, &c).extra_broken, 1);
}
#[test]
fn cooldown_and_toggles_are_per_player_and_original_mining_is_never_cancelled() {
    let mut rt = Runtime::default();
    let c = Config::default();
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    assert_eq!(rt.mine(&mut w, &c, [0, 0, 0]).unwrap().extra_broken, 1);
    for tick in [0, 19] {
        let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
        w.player.tick = tick;
        assert_eq!(rt.mine(&mut w, &c, [0, 0, 0]).unwrap().extra_broken, 0);
        assert_eq!(w.calls.len(), 1);
    }
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    w.player.actor = [0, 0, 0, 2];
    assert_eq!(rt.mine(&mut w, &c, [0, 0, 0]).unwrap().extra_broken, 1);
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    w.player.tick = 20;
    assert_eq!(rt.mine(&mut w, &c, [0, 0, 0]).unwrap().extra_broken, 1);
    rt.set_enabled([0, 0, 0, 1], false);
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    w.player.tick = 100;
    assert_eq!(rt.mine(&mut w, &c, [0, 0, 0]).unwrap().extra_broken, 0);
    rt.set_enabled(w.player.actor, true);
    w.blocks.insert([0, 0, 0], block("minecraft:iron_ore", 1));
    assert_eq!(rt.mine(&mut w, &c, [0, 0, 0]).unwrap().extra_broken, 1);
}
#[test]
fn group_tools_overrides_and_separate_group_mining_have_defined_precedence() {
    let mut c = Config::default();
    let g = c.groups.iter_mut().find(|g| g.id == "iron-ore").unwrap();
    g.tools.insert("minecraft:diamond_pickaxe".into());
    g.overrides.max_chain = Some(2);
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0], [2, 0, 0]]);
    assert_eq!(mine(&mut w, &c).extra_broken, 0);
    w.blocks.insert([0, 0, 0], block("minecraft:iron_ore", 1));
    w.tool.as_mut().unwrap().item = "minecraft:diamond_pickaxe".into();
    assert_eq!(mine(&mut w, &c).extra_broken, 1);
    let mut w = World::new(&[[0, 0, 0], [1, 0, 0]]);
    w.blocks
        .insert([1, 0, 0], block("minecraft:deepslate_iron_ore", 2));
    let mut c = Config::default();
    c.settings.separate_group_mining = true;
    assert_eq!(mine(&mut w, &c).extra_broken, 0);
}
#[test]
fn malformed_unknown_and_unbounded_configuration_fails_closed() {
    assert!(Config::default().validate().is_ok());
    for source in [
        r#"{"settings":{"maxChain":0}}"#,
        r#"{"settings":{"searchRadius":100}}"#,
        r#"{"settings":{"hungerPerBlock":-1}}"#,
        r#"{"settings":{"delay":5}}"#,
        r#"{"groups":[{"id":"x","blocks":["no-namespace"]}]}"#,
        r#"{"groups":[{"id":"x","blocks":["minecraft:stone"]},{"id":"x","blocks":["minecraft:dirt"]}]}"#,
    ] {
        assert!(Config::from_json(source).is_err(), "{source}");
    }
    let c = Config::default();
    let json = serde_json::to_string(&c).unwrap();
    assert!(Config::from_json(&json).is_ok());
}
#[test]
fn coordinates_at_integer_limits_do_not_wrap_to_other_chunks() {
    let p = [i32::MAX, 0, i32::MIN];
    let w = World::new(&[p]);
    assert_eq!(
        preview(
            &w,
            p,
            &["minecraft:iron_ore".into()].into(),
            &Settings::default()
        )
        .len(),
        1
    );
}
