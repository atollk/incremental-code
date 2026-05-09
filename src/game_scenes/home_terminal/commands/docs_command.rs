use crate::game_scenes::base::SceneSwitch;
use crate::game_state::with_game_state;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::text::{Line, Text};
use ratatui_widgets::paragraph::Paragraph;

pub(super) fn docs_cmd() -> Box<dyn RunningCommand<SceneSwitch>> {
    let (unlock_print, unlock_sleep, unlock_brk) = with_game_state(|gs| {
        (
            gs.upgrades.unlock_print.value(),
            gs.upgrades.unlock_sleep.value(),
            gs.upgrades.unlock_brk.value(),
        )
    });

    let mut lines: Vec<Line> = Vec::new();
    macro_rules! ln {
        () => {
            lines.push(Line::from(""))
        };
        ($s:literal) => {
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
    ln!("VARIABLES");
    ln!("  x := 5;         declare");
    ln!("  x = 10;         reassign");
    ln!("  a[0]            index into a list or dict");
    ln!();
    ln!("TYPES");
    ln!("  42  1_000_000   integer");
    ln!("  3.14  .5        float");
    ln!("  \"hello\"         string");
    ln!("  True  False     boolean");
    ln!("  None            null");
    ln!("  [1, 2, 3]       list");
    ln!("  {\"k\": \"v\"}      dict");
    ln!();
    ln!("OPERATORS");
    ln!("  + - * / %         arithmetic");
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
    ln!("LOOPS");
    ln!("  loop:");
    ln!("    break;");
    ln!("  end");
    ln!("  break and continue are supported inside loops");
    ln!();
    ln!("FUNCTIONS");
    ln!("  def add(a, b):");
    ln!("    return a + b;");
    ln!("  end");
    ln!("  return; with no value returns None");

    if unlock_print {
        ln!();
        ln!("BUILT-IN: print");
        ln!("  print(value)");
        ln!("  Converts value to a string and displays it.");
        ln!("  Earns silver proportional to the number of characters.");
    }

    if unlock_sleep {
        ln!();
        ln!("BUILT-IN: sleep");
        ln!("  sleep(seconds)");
        ln!("  Pauses execution for the given duration.");
        ln!("  Earns gold proportional to the sleep time.");
    }

    if unlock_brk {
        ln!();
        ln!("BUILT-IN: brk");
        ln!("  brk()");
        ln!("  Triggers a breakpoint, slowing down execution.");
        ln!("  Earns diamond each time a breakpoint is hit.");
    }

    Box::new(ParagraphCmd::new(Paragraph::new(Text::from(lines))))
}
