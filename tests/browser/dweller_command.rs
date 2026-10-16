use super::dweller;
use std::process::Command;

pub fn dweller_command() -> Command {
    let mut command = Command::new(dweller());

    if std::env::var_os("DWELLER_TEST_REAL_WINDOW").is_some_and(|v| v == "1") {
        command.env("DWELLER_BACKGROUND", "1");
    } else {
        command.env("SDL_VIDEODRIVER", "dummy");
    }

    command
}
