fn guest_command(game: &mut crate::game::Game<'_>, host: &mut crate::game::lan::LanHost, conn: u32, command: &str) -> core::ops::ControlFlow<()> {
    if command.trim_start_matches('/').split_whitespace().next() != Some("veinminer") {
        return core::ops::ControlFlow::Continue(());
    }
    if let Some(permission) = host.hub.guest(conn).map(|g| g.permission) {
        host.commands.push(crate::game::lan_actions::GuestCommand {
            conn, command: command.trim_start_matches('/').to_owned(), permission,
            integrated: host.hub.config.integrated, quiet: false,
        });
    }
    let _ = game;
    core::ops::ControlFlow::Break(())
}
