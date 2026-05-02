use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::commands::code_command::code_cmd;
use crate::game_scenes::home_terminal::commands::compile_command::compile_cmd;
use crate::game_scenes::home_terminal::commands::exit_command::exit_cmd;
use crate::game_scenes::home_terminal::commands::help_command::help_cmd;
use crate::widgets::terminal::RunningCommand;

mod code_command;
mod compile_command;
mod exit_command;
mod help_command;
mod run_command;
mod unknown_command;
mod upgrades_command;

use crate::game_scenes::home_terminal::commands::run_command::run_cmd;
use crate::game_scenes::home_terminal::commands::upgrades_command::upgrades_cmd;
pub use unknown_command::unknown_cmd;

/// A terminal command entry: its name, help text, and a factory that creates a runner for it.
pub struct Command {
    pub(crate) name: &'static str,
    help_description: &'static str,
    pub(crate) runner: fn() -> Box<dyn RunningCommand<SceneSwitch>>,
}

/// Returns the full list of available terminal commands.
pub fn command_list() -> Vec<Command> {
    vec![
        Command {
            name: "help",
            help_description: "Displays this help text",
            runner: help_cmd,
        },
        Command {
            name: "exit",
            help_description: "Exits the game",
            runner: exit_cmd,
        },
        Command {
            name: "code",
            help_description: "Opens the code editor to write or modify your program",
            runner: code_cmd,
        },
        Command {
            name: "compile",
            help_description: "Compiles the program code to make it executable",
            runner: compile_cmd,
        },
        Command {
            name: "run",
            help_description: "Runs the program code after compiling",
            runner: run_cmd,
        },
        Command {
            name: "upgrades",
            help_description: "Opens the upgrade tree",
            runner: upgrades_cmd,
        },
    ]
}
