use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::home_terminal::commands::code_command::code_cmd;
use crate::game_scenes::home_terminal::commands::compile_command::compile_cmd;
use crate::game_scenes::home_terminal::commands::exit_command::exit_cmd;
use crate::game_scenes::home_terminal::commands::help_command::help_cmd;
use crate::widgets::terminal::RunningCommand;

mod code_command;
mod compile_command;
mod docs_command;
mod exit_command;
mod help_command;
mod reboot_command;
mod reset_command;
mod run_command;
mod save_command;
mod unknown_command;
mod upgrades_command;
mod volume_command;

use crate::game_scenes::home_terminal::HomeTerminalScene;
use crate::game_scenes::home_terminal::commands::docs_command::docs_cmd;
use crate::game_scenes::home_terminal::commands::reboot_command::reboot_cmd;
use crate::game_scenes::home_terminal::commands::reset_command::reset_cmd;
use crate::game_scenes::home_terminal::commands::run_command::run_cmd;
use crate::game_scenes::home_terminal::commands::save_command::save_cmd;
use crate::game_scenes::home_terminal::commands::upgrades_command::upgrades_cmd;
use crate::game_scenes::home_terminal::commands::volume_command::volume_cmd;
use crate::game_state::with_game_state;
pub use unknown_command::unknown_cmd;

/// A terminal command entry: its name, help text, and a factory that creates a runner for it.
pub struct Command {
    pub(crate) name: &'static str,
    help_description: &'static str,
    pub(crate) runner: fn(&HomeTerminalScene) -> Box<dyn RunningCommand<SceneSwitch>>,
}

/// Returns the full list of available terminal commands.
pub fn command_list() -> Vec<Command> {
    let (unlock_code, unlock_music, unlock_reboot) = with_game_state(|game_state| {
        (
            game_state.upgrades.unlock_code.value(),
            game_state.upgrades.unlock_music.value(),
            game_state.upgrades.unlock_reboot.value(),
        )
    });

    let mut commands = Vec::new();
    commands.push(Command {
        name: "help",
        help_description: "Displays this help text",
        runner: |_| help_cmd(),
    });
    commands.push(Command {
        name: "save",
        help_description: "Saves the game. Will be loaded automatically on the next startup.",
        runner: |_| save_cmd(),
    });
    if unlock_reboot {
        commands.push(Command {
            name: "reboot",
            help_description: "Reset all upgrades but gain additional currency",
            runner: |_| reboot_cmd(),
        });
    }
    if unlock_code {
        commands.push(Command {
            name: "docs",
            help_description: "Explanation of the coding language",
            runner: |scene| docs_cmd(scene.height),
        });
        commands.push(Command {
            name: "code",
            help_description: "Opens the code editor to write or modify your program",
            runner: |_| code_cmd(),
        });
        commands.push(Command {
            name: "compile",
            help_description: "Compiles the program code to make it executable",
            runner: |_| compile_cmd(),
        });
        commands.push(Command {
            name: "run",
            help_description: "Runs the program code after compiling",
            runner: |_| run_cmd(),
        });
    }
    commands.push(Command {
        name: "upgrades",
        help_description: "Opens the upgrade tree",
        runner: |_| upgrades_cmd(),
    });
    if unlock_music {
        commands.push(Command {
            name: "volume",
            help_description: "Control the music volume",
            runner: |_| volume_cmd(),
        })
    }
    commands.push(Command {
        name: "reset",
        help_description: "Resets the game, removing any save data",
        runner: |_| reset_cmd(),
    });
    commands.push(Command {
        name: "exit",
        help_description: "Exits the game",
        runner: |_| exit_cmd(),
    });
    commands
}
