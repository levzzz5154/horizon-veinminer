// Native adapter for Horizon 028f60881678a9a0e3e198f693a82858fa475c76.
use super::{Game, Result};
mod engine {
    include!("veinminer_engine.rs");
}
use engine::{Block, Config, Host, Player, Position, Runtime, Tool};

#[derive(Default)]
pub(super) struct State {
    runtime: Runtime,
    config: Option<Config>,
}

thread_local! {
    static STATES: std::cell::RefCell<std::collections::BTreeMap<std::path::PathBuf, State>> = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}
fn with_state<T>(game: &mut Game<'_>, run: impl FnOnce(&mut Game<'_>, &mut State) -> T) -> T {
    struct Lease {
        key: std::path::PathBuf,
        state: State,
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            STATES.with(|states| {
                states
                    .borrow_mut()
                    .insert(self.key.clone(), std::mem::take(&mut self.state));
            });
        }
    }
    let key = game.world_root.clone();
    let state = STATES.with(|states| states.borrow_mut().remove(&key).unwrap_or_default());
    let mut lease = Lease { key, state };
    run(game, &mut lease.state)
}

fn config_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HORIZON_VEINMINER_CONFIG").map(Into::into)
}
fn load_config() -> std::result::Result<Config, String> {
    match config_path() {
        Some(path) => Config::from_json(
            &std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        ),
        None => Ok(Config::default()),
    }
}

struct Native<'g, 'r> {
    game: &'g mut Game<'r>,
}
impl Host for Native<'_, '_> {
    type Error = Box<dyn std::error::Error>;
    fn player(&mut self) -> Player {
        Player {
            actor: self.game.acting_player_uuid(),
            name: self
                .game
                .lan_acting_name
                .as_deref()
                .unwrap_or(super::chat::PLAYER_NAME)
                .to_owned(),
            tick: self.game.world.game_time,
            survival: self.game.remote.is_none()
                && self.game.player.is_some()
                && self.game.may_build()
                && !self.game.creative,
            sneaking: self
                .game
                .keys
                .contains(&winit::keyboard::KeyCode::ShiftLeft),
        }
    }
    fn block(&self, p: Position) -> Option<Block> {
        if p[1] < self.game.world.shape.min_y
            || p[1] >= self.game.world.shape.max_y()
            || !self.game.world.loaded(p[0], p[2])
        {
            return None;
        }
        let state = self.game.world.block(p[0], p[1], p[2]);
        let facts = self.game.res.facts().get(state);
        Some(Block {
            name: facts.name.to_string(),
            state: state.0,
            breakable: facts.hardness >= 0.0 && !self.game.world.is_air(state),
        })
    }
    fn tool(&self) -> Option<Tool> {
        let stack = self.game.held()?;
        let max = self
            .game
            .res
            .items()
            .get(&stack.item)
            .map_or(0, |i| i.max_damage);
        Some(Tool {
            item: stack.item.to_string(),
            remaining: (max > 0).then_some(max.saturating_sub(stack.damage)),
        })
    }
    fn can_harvest(&self, block: &Block) -> bool {
        let facts = self
            .game
            .res
            .facts()
            .get(horizon_world::registry::BlockStateId(block.state));
        !facts.requires_tool
            || self
                .game
                .res
                .items()
                .correct_for_drops(self.game.held(), &block.name)
    }
    fn has_food(&self) -> bool {
        self.game
            .player
            .as_ref()
            .is_some_and(|p| p.health.food.food > 0 || p.health.food.saturation > 0.0)
    }
    fn mine(&mut self, p: Position, preserve: bool) -> Result<bool> {
        let before = self.block(p);
        let held = preserve.then(|| self.game.inventory[self.game.selected].clone());
        // Temporarily remove pre-existing damage, then restore damage (and a tool
        // destroyed by this one operation) without changing other mining effects.
        if preserve && let Some(stack) = self.game.inventory[self.game.selected].as_mut() {
            stack.damage = 0;
        }
        let result = self.game.break_block_as_player(p);
        if let Some(original) = held {
            match (self.game.inventory[self.game.selected].as_mut(), original) {
                (Some(now), Some(was)) if now.item == was.item => now.damage = was.damage,
                (None, Some(was)) => self.game.inventory[self.game.selected] = Some(was),
                _ => {}
            }
        }
        result?;
        Ok(before.is_some() && self.block(p) != before)
    }
    fn exhaust(&mut self, amount: f32) {
        if amount > 0.0
            && let Some(p) = self.game.player.as_mut()
        {
            p.cause_food_exhaustion(amount);
        }
    }
}
impl Game<'_> {
    pub(super) fn veinminer_break_block(&mut self, pos: Position) -> Result<()> {
        with_state(self, |game, state| {
            if state.config.is_none() {
                state.config = Some(load_config().unwrap_or_else(|error| {
                    eprintln!("Veinminer disabled: {error}");
                    let mut config = Config::default();
                    config.settings.enabled = false;
                    config
                }));
            }
            let config = state.config.as_ref().unwrap();
            state
                .runtime
                .mine(&mut Native { game }, config, pos)
                .map(|_| ())
        })
    }
    pub(super) fn veinminer_command(&mut self, command: &str) -> Option<bool> {
        with_state(self, |game, state| {
            let mut words = command.split_whitespace();
            if words.next()? != "veinminer" {
                return None;
            }
            let args: Vec<_> = words.collect();
            let actor = game.acting_player_uuid();
            let message = match args.as_slice() {
                [] | ["status"] => format!(
                    "Veinminer: {}",
                    if state.runtime.enabled(actor) {
                        "on"
                    } else {
                        "off"
                    }
                ),
                ["toggle"] | ["on"] | ["off"] => {
                    let on = match args[0] {
                        "on" => true,
                        "off" => false,
                        _ => !state.runtime.enabled(actor),
                    };
                    state.runtime.set_enabled(actor, on);
                    format!("Veinminer: {}", if on { "on" } else { "off" })
                }
                ["reload"] => {
                    if game.ui.registries.permission < 2 {
                        game.command_failure(horizon_client::text::Text::literal(
                            "Veinminer reload requires permission level 2",
                        ));
                        return Some(false);
                    }
                    match load_config() {
                        Ok(config) => {
                            state.config = Some(config);
                            "Veinminer configuration reloaded".into()
                        }
                        Err(error) => {
                            game.command_failure(horizon_client::text::Text::literal(error));
                            return Some(false);
                        }
                    }
                }
                _ => {
                    game.command_failure(horizon_client::text::Text::literal(
                        "Usage: /veinminer [status|toggle|on|off|reload]",
                    ));
                    return Some(false);
                }
            };
            game.system_message(horizon_client::text::Text::literal(message));
            Some(true)
        })
    }
}

#[cfg(test)]
mod engine_tests {
    use super::engine as horizon_veinminer;
    include!("veinminer_tests.rs");
}
