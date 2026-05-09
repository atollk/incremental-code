use crate::backend::events::Event;
use crate::backend::input::{KeyCode, KeyEventKind};
use crate::game_scenes::base::SceneSwitch;
use crate::game_state::with_game_state;
use crate::widgets::terminal::{ParagraphCmd, RunningCommand};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::{Line, Text};
use ratatui_core::widgets::Widget;
use ratatui_widgets::paragraph::Paragraph;
use std::cell::Cell;
use std::cmp::min;
use std::time::Duration;

pub(super) fn docs_cmd(height: u16) -> Box<dyn RunningCommand<SceneSwitch>> {
    Box::new(DocsCommand::new(height))
}

fn get_docs_lines() -> Vec<Line<'static>> {
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
        DocsCommand {
            scroll_y: Cell::new(0),
            exit: false,
            lines: get_docs_lines(),
            height,
        }
    }
}

impl RunningCommand<SceneSwitch> for DocsCommand {
    fn is_done(&self) -> bool {
        self.exit
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
        min(self.height, self.lines.len() as u16) - 1
    }

    fn get_metadata(&self) -> SceneSwitch {
        SceneSwitch::NoSwitch
    }
}
