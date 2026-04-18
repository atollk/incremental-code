use std::sync::OnceLock;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use web_time::Instant;

/// A blinking cursor that paints a single cell at `(x, y)`.
pub struct BlinkingCursor {
    x: u16,
    y: u16,
    blink_on: bool,
    style: Style,
    symbol: Option<&'static str>,
}

impl BlinkingCursor {
    /// Create a cursor at the given buffer coordinates.
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            blink_on: true,
            style: Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
            symbol: None,
        }
    }

    /// Derive blink state from an `Instant`. `period_ms` is the full
    /// on+off cycle; the cursor is visible for the first half.
    pub fn with_blink(mut self, period_ms: u64) -> Self {
        let elapsed = epoch().elapsed().as_millis() as u64;
        let half = (period_ms / 2).max(1);
        self.blink_on = (elapsed / half) % 2 == 0;
        self
    }

    /// Set blink state directly (e.g. if you drive it from your own tick).
    pub fn blink_on(mut self, on: bool) -> Self {
        self.blink_on = on;
        self
    }

    /// Override the cursor style. Default is inverted white-on-black bold.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Override the glyph drawn. By default the existing cell's symbol is
    /// preserved and only the style is changed (block-over-character look).
    /// Set this to e.g. "_" or "▏" for underscore / bar cursors.
    pub fn symbol(mut self, symbol: &'static str) -> Self {
        self.symbol = Some(symbol);
        self
    }
}

impl Widget for BlinkingCursor {
    fn render(self, _area: Rect, buf: &mut Buffer) {
        if !self.blink_on {
            return;
        }
        if self.x >= buf.area.right() || self.y >= buf.area.bottom() {
            return;
        }

        let cell = &mut buf[(self.x, self.y)];
        if let Some(sym) = self.symbol {
            cell.set_symbol(sym);
        } else if cell.symbol().is_empty() || cell.symbol() == " " {
            // Nothing under the cursor — paint a space so the bg color shows.
            cell.set_symbol(" ");
        }
        cell.set_style(self.style);
    }
}

fn epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}