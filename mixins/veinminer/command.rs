fn command_head(game: &mut crate::game::Game<'_>, command: &str) -> core::ops::ControlFlow<bool> {
    match game.veinminer_command(command) {
        Some(result) => core::ops::ControlFlow::Break(result),
        None => core::ops::ControlFlow::Continue(()),
    }
}
