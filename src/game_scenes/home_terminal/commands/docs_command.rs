use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_scenes::logic::verification::float_decimals;
use crate::game_state::{BRONZE_SYMBOL, CodeStatementLevels, with_game_state};
use crate::widgets::terminal::RunningCommand;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::Widget;
use std::cell::Cell;
use std::cmp::min;
use std::time::Duration;

pub(super) fn docs_cmd(height: u16) -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(DocsCommand::new(height))
}

fn get_docs_lines() -> Vec<Line<'static>> {
    let (
        unlock_print,
        unlock_sleep,
        unlock_brk,
        statements,
        (strings_allowed, max_int),
        bronze_per_instruction,
        bronze_per_instruction_next_value,
    ) = with_game_state(|gs| {
        (
            gs.upgrades.unlock_print.value(),
            gs.upgrades.unlock_sleep.value(),
            gs.upgrades.unlock_brk.value(),
            gs.upgrades.statements.value(),
            gs.upgrades.literals.value(),
            gs.upgrades.bronze_per_instruction.value(),
            gs.upgrades.bronze_per_instruction.next_value(),
        )
    });

    // Explain `n` once the last constant-only level was bought,
    // i.e. when the next level (or the current one) has an exponent.
    let bronze_n = bronze_per_instruction.1 > 0.
        || bronze_per_instruction_next_value.is_some_and(|(_, exponent)| exponent > 0.);

    // Mirrors the gating in `verification::verify_stmt`.
    let allows_loops = !matches!(statements, CodeStatementLevels::None);
    let allows_nested_loops = !matches!(
        statements,
        CodeStatementLevels::None | CodeStatementLevels::SimpleLoops
    );
    let allows_functions = matches!(
        statements,
        CodeStatementLevels::Functions
            | CodeStatementLevels::PureFunctions
            | CodeStatementLevels::SingleRecursion
            | CodeStatementLevels::MultiRecursion
    );
    let allows_pure_functions = matches!(
        statements,
        CodeStatementLevels::PureFunctions
            | CodeStatementLevels::SingleRecursion
            | CodeStatementLevels::MultiRecursion
    );
    let allows_multiplication = matches!(
        statements,
        CodeStatementLevels::Multiplication
            | CodeStatementLevels::PureFunctions
            | CodeStatementLevels::SingleRecursion
            | CodeStatementLevels::MultiRecursion
    );
    let max_recursion = match statements {
        CodeStatementLevels::SingleRecursion => 1,
        CodeStatementLevels::MultiRecursion => usize::MAX,
        _ => 0,
    };

    let mut lines: Vec<Line> = Vec::new();
    macro_rules! ln {
        () => {
            lines.push(Line::from(""))
        };
        ($s:literal) => {
            lines.push(Line::from($s))
        };
        ($s:expr) => {
            lines.push(Line::from($s))
        };
    }

    ln!("NotPython is a small programming language.");
    ln!();

    ln!("SYNTAX RULES");
    ln!("  Simple statements end with ;");
    ln!("  Block headers (if, loop, def) end with :");
    ln!("  Blocks close with end");
    ln!();
    ln!("RESOURCES");
    ln!(format!(
        "  Every instruction executed yields {BRONZE_SYMBOL}."
    ));
    ln!();
    ln!("VARIABLES");
    ln!("  x := 5;         declare");
    ln!("  x = 10;         reassign");
    ln!("  a[0]            index into a list or dict");
    ln!();
    ln!("TYPES");
    ln!(format!(
        "  {:<16}integer",
        format!("-{max_int} to {max_int}")
    ));
    match float_decimals(max_int as i64) {
        None => {}
        Some(1) => ln!("  0.1  .5         float (1 decimal)"),
        Some(d) => ln!(format!("  0.25  .5        float ({d} decimals)")),
    }
    if strings_allowed {
        ln!("  \"hello\"         string");
    }
    ln!("  True  False     boolean");
    ln!("  None            null");
    ln!("  [1, 2, 3]       list");
    ln!("  {\"k\": \"v\"}      dict");
    ln!();
    ln!("OPERATORS");
    if allows_multiplication {
        ln!("  + - * / %         arithmetic");
    } else {
        ln!("  + - %             arithmetic");
    }
    ln!("  == != < > <= >=   comparison");
    ln!("  and  or  not      boolean");
    ln!("  in                membership test");
    ln!();
    ln!("CONDITIONALS");
    ln!("  if x > 0:");
    ln!("    pass;");
    ln!("  elif x == 0:");
    ln!("    pass;");
    ln!("  else:");
    ln!("    pass;");
    ln!("  end");
    ln!();

    if allows_loops {
        ln!("LOOPS");
        ln!("  loop:");
        ln!("    break;");
        ln!("  end");
        ln!("  break and continue are supported inside loops");
        if allows_nested_loops {
            ln!("  loops may be nested");
        } else {
            ln!("  loops cannot be nested yet");
        }
        ln!();
    }

    if allows_functions {
        ln!("FUNCTIONS");
        ln!("  def add(a, b):");
        ln!("    return a + b;");
        ln!("  end");
        ln!("  return; with no value returns None");

        if allows_pure_functions {
            ln!();
            ln!("MODIFIER: pure");
            ln!("  def pure add(a, b):");
            ln!("    return a + b;");
            ln!("  end");
            ln!("  Caches the result per input for faster compile and run times.");
            ln!("  May only access its own local variables, not outer-scope ones.");
        }

        ln!();
        ln!(match max_recursion {
            0 => "  functions cannot call themselves yet".to_string(),
            1 => "  functions may call themselves once (single recursion)".to_string(),
            _ => "  functions may recurse freely".to_string(),
        });
    }

    if unlock_sleep {
        ln!();
        ln!("BUILT-IN: sleep");
        ln!("  sleep(seconds)");
        ln!("  Pauses execution for the given number of seconds.");
        ln!("  The wait is scaled by the current instruction speed,");
        ln!("  so it can be shorter than the argument.");
        ln!("  Earns silver based on the seconds passed to sleep.");
        ln!();
        ln!("  Sleep Speed Reset:");
        ln!("  Instruction speed can ramp up with the number of");
        ln!("  instructions run since the last sleep. After each sleep,");
        ln!("  the speed boost gained so far is raised to the power of");
        ln!("  the Sleep Speed Reset value: ^0 loses the boost fully,");
        ln!("  \"none\" keeps all of it.");
    }

    if unlock_print {
        ln!();
        ln!("BUILT-IN: print");
        ln!("  print(value)");
        ln!("  Converts value to a string and displays it.");
        ln!("  Earns gold based on the length of the longest string");
        ln!("  printed during a run; shorter prints do not add up.");
    }

    if unlock_brk {
        ln!();
        ln!("BUILT-IN: brk");
        ln!("  brk()");
        ln!("  Triggers a breakpoint, slowing down execution.");
        ln!("  Earns diamond each time a breakpoint is hit.");
    }

    if bronze_n {
        ln!();
        ln!("BRONZE");
        ln!("  Every executed instruction earns bronze.");
        ln!("  In the upgrade names, n is the number of instructions");
        ln!("  executed in the run: \"n ^ 2\" means the bronze earned by");
        ln!("  the n-th instruction grows like n squared.");
    }

    lines
}

struct DocsCommand {
    scroll_y: Cell<u16>,
    exit: bool,
    lines: Vec<Line<'static>>,
    height: u16,
}

impl DocsCommand {
    fn new(height: u16) -> DocsCommand {
        let mut lines = get_docs_lines();
        if lines.len() >= height as usize {
            lines.push(Line::from("[Up/Down] scroll   [Enter/Esc] close"));
        }
        DocsCommand {
            scroll_y: Cell::new(0),
            exit: false,
            lines,
            height,
        }
    }
}

impl RunningCommand<SceneSwitch> for DocsCommand {
    fn is_done(&self) -> bool {
        // Everything fits on screen: nothing to scroll, so don't wait for input.
        self.exit || self.lines.len() < self.height as usize
    }

    fn update(&mut self, events: &[Event], _time_delta: Duration) {
        for event in events {
            if let Event::KeyEvent(key_event) = event {
                let pressed = key_event.kind == KeyEventKind::Press;
                let repeated = key_event.kind == KeyEventKind::Repeat;
                match key_event.code {
                    KeyCode::Enter | KeyCode::Esc if pressed => self.exit = true,
                    KeyCode::Up | KeyCode::Down if pressed || repeated => {
                        let up = key_event.code == KeyCode::Up;
                        let scroll_y = self.scroll_y.get_mut();
                        if up {
                            *scroll_y = scroll_y.saturating_sub(1);
                        } else {
                            *scroll_y = min(*scroll_y + 1, self.lines.len() as u16);
                        }
                    }
                    _ => {}
                };
            }
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Clip scroll_y
        while let y = self.scroll_y.get()
            && y + area.height > self.lines.len() as u16
            && y > 0
        {
            self.scroll_y.set(self.scroll_y.get() - 1);
        }

        let y = self.scroll_y.get() as usize;
        let sub_lines = self.lines[y..y + area.height as usize].to_vec();
        Text::from(sub_lines).render(area, buf);
    }

    fn height(&self, _columns: u16) -> u16 {
        min(self.height.saturating_sub(1), self.lines.len() as u16)
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::NoSwitch
    }
}
