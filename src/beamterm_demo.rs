//! Ratatui demo with tachyonfx effects, rendered via beamterm-core on native OpenGL 3.3.
//!
//! Ported from the ratzilla demo example, adapted for desktop windowing.
//!
//! Run with:
//! ```sh
//! cargo run -p demo
//! ```

use std::time::Instant;

use crate::backend::events::Event;
use crate::backend::input::KeyCode;
use crate::basic_terminal_app::App;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{self, Span},
    widgets::{
        canvas::{self, Canvas, Circle, Map, MapResolution, Rectangle}, Axis, BarChart, Block, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        ListState, Paragraph, Row, Sparkline, Table, Tabs,
        Wrap,
    },
    Frame,
};
use tachyonfx::{
    fx::*, CellFilter, ColorSpace, Duration, Effect, EffectManager, EffectTimer,
    Interpolation::*, Motion, RangeSampler, SimpleRng,
};

pub struct BeamtermDemo {
    title: &'static str,
    should_quit: bool,
    tabs: TabsState<'static>,
    show_chart: bool,
    progress: f64,
    sparkline: Signal<RandomSignal>,
    tasks: StatefulList<&'static str>,
    logs: StatefulList<(&'static str, &'static str)>,
    signals: Signals,
    barchart: Vec<(&'static str, u64)>,
    servers: Vec<Server<'static>>,
    enhanced_graphics: bool,
    effects: EffectManager<EffectKey>,
    last_frame: Instant,
}

impl BeamtermDemo {
    fn new(title: &'static str, enhanced_graphics: bool) -> Self {
        let mut rand_signal = RandomSignal::new(0, 100);
        let sparkline_points = rand_signal.by_ref().take(300).collect();
        let mut sin_signal = SinSignal::new(0.2, 3.0, 18.0);
        let sin1_points = sin_signal.by_ref().take(100).collect();
        let mut sin_signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let sin2_points = sin_signal2.by_ref().take(200).collect();

        let mut effects = EffectManager::default();
        effects.add_effect(fx_startup());
        effects.add_effect(fx_pulsate_selected_tab());
        BeamtermDemo {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Tab0", "Tab1", "Tab2"]),
            show_chart: true,
            progress: 0.0,
            sparkline: Signal {
                source: rand_signal,
                points: sparkline_points,
                tick_rate: 1,
            },
            tasks: StatefulList::with_items(TASKS.to_vec()),
            logs: StatefulList::with_items(LOGS.to_vec()),
            signals: Signals {
                sin1: Signal {
                    source: sin_signal,
                    points: sin1_points,
                    tick_rate: 5,
                },
                sin2: Signal {
                    source: sin_signal2,
                    points: sin2_points,
                    tick_rate: 10,
                },
                window: [0.0, 20.0],
            },
            barchart: EVENTS.to_vec(),
            servers: vec![
                Server {
                    name: "NorthAmerica-1",
                    location: "New York City",
                    coords: (40.71, -74.00),
                    status: "Up",
                },
                Server {
                    name: "Europe-1",
                    location: "Paris",
                    coords: (48.85, 2.35),
                    status: "Failure",
                },
                Server {
                    name: "SouthAmerica-1",
                    location: "São Paulo",
                    coords: (-23.54, -46.62),
                    status: "Up",
                },
                Server {
                    name: "Asia-1",
                    location: "Singapore",
                    coords: (1.35, 103.86),
                    status: "Up",
                },
            ],
            enhanced_graphics,
            effects,
            last_frame: Instant::now(),
        }
    }

    fn on_tick(&mut self) -> Duration {
        self.progress += 0.001;
        if self.progress > 1.0 {
            self.progress = 0.0;
        }

        self.sparkline.on_tick();
        self.signals.on_tick();

        let log = self.logs.items.pop().unwrap();
        self.logs.items.insert(0, log);

        let event = self.barchart.pop().unwrap();
        self.barchart.insert(0, event);

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame).as_millis() as u32;
        self.last_frame = now;

        Duration::from_millis(elapsed)
    }

    fn add_transition_tab_effect(&mut self) {
        let effect = fx_change_tab();
        self.effects.add_unique_effect(EffectKey::ChangeTab, effect);
    }
}


impl App for BeamtermDemo {
    fn frame(
        &mut self,
        events: &[Event],
        frame: &mut Frame,
    ) -> anyhow::Result<bool> {
        let tick_duration = self.on_tick();
        for event in events {
            match event {
                Event::KeyEvent(key) => match key.code {
                    KeyCode::Left => {
                        self.tabs.previous();
                        self.add_transition_tab_effect();
                    }
                    KeyCode::Right => {
                        self.tabs.next();
                        self.add_transition_tab_effect();
                    }
                    KeyCode::Up => {
                        self.tasks.previous();
                    }
                    KeyCode::Down => {
                        self.tasks.next();
                    }
                    KeyCode::Char(c) => {
                        match c {
                            'q' => self.should_quit = true,
                            't' => self.show_chart = !self.show_chart,
                            _ => {}
                        }
                    }
                    _ => {}
                },
                Event::MouseEvent(_) => {}
            }
        }
        ui_draw(tick_duration, frame, self);
        Ok(self.should_quit)
    }
}

impl Default for BeamtermDemo {
    fn default() -> Self {
        BeamtermDemo::new("Beamterm Demo", true)
    }
}

const TASKS: [&str; 24] = [
    "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
    "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
    "Item20", "Item21", "Item22", "Item23", "Item24",
];

const LOGS: [(&str, &str); 26] = [
    ("Event1", "INFO"),
    ("Event2", "INFO"),
    ("Event3", "CRITICAL"),
    ("Event4", "ERROR"),
    ("Event5", "INFO"),
    ("Event6", "INFO"),
    ("Event7", "WARNING"),
    ("Event8", "INFO"),
    ("Event9", "INFO"),
    ("Event10", "INFO"),
    ("Event11", "CRITICAL"),
    ("Event12", "INFO"),
    ("Event13", "INFO"),
    ("Event14", "INFO"),
    ("Event15", "INFO"),
    ("Event16", "INFO"),
    ("Event17", "ERROR"),
    ("Event18", "ERROR"),
    ("Event19", "INFO"),
    ("Event20", "INFO"),
    ("Event21", "WARNING"),
    ("Event22", "INFO"),
    ("Event23", "INFO"),
    ("Event24", "WARNING"),
    ("Event25", "INFO"),
    ("Event26", "INFO"),
];

const EVENTS: [(&str, u64); 24] = [
    ("B1", 9),
    ("B2", 12),
    ("B3", 5),
    ("B4", 8),
    ("B5", 2),
    ("B6", 4),
    ("B7", 5),
    ("B8", 9),
    ("B9", 14),
    ("B10", 15),
    ("B11", 1),
    ("B12", 0),
    ("B13", 4),
    ("B14", 6),
    ("B15", 4),
    ("B16", 6),
    ("B17", 4),
    ("B18", 7),
    ("B19", 13),
    ("B20", 8),
    ("B21", 11),
    ("B22", 9),
    ("B23", 3),
    ("B24", 5),
];

#[derive(Clone)]
struct RandomSignal {
    lower: u32,
    upper: u32,
    rng: SimpleRng,
}

impl RandomSignal {
    fn new(lower: u64, upper: u64) -> Self {
        Self {
            lower: lower as u32,
            upper: upper as u32,
            rng: SimpleRng::default(),
        }
    }
}

impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.rng.gen_range(self.lower..self.upper) as u64)
    }
}

#[derive(Clone)]
struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    const fn new(interval: f64, period: f64, scale: f64) -> Self {
        Self {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

struct TabsState<'a> {
    titles: Vec<&'a str>,
    index: usize,
}

impl<'a> TabsState<'a> {
    const fn new(titles: Vec<&'a str>) -> Self {
        Self { titles, index: 0 }
    }
    fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

struct Signal<S: Iterator> {
    source: S,
    points: Vec<S::Item>,
    tick_rate: usize,
}

impl<S: Iterator> Signal<S> {
    fn on_tick(&mut self) {
        self.points.drain(0..self.tick_rate);
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

struct Signals {
    sin1: Signal<SinSignal>,
    sin2: Signal<SinSignal>,
    window: [f64; 2],
}

impl Signals {
    fn on_tick(&mut self) {
        self.sin1.on_tick();
        self.sin2.on_tick();
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

struct Server<'a> {
    name: &'a str,
    location: &'a str,
    coords: (f64, f64),
    status: &'a str,
}

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
enum EffectKey {
    #[default]
    ChangeTab,
}

// ── Effects ─────────────────────────────────────────────────────────

const BG_COLOR: Color = Color::from_u32(0x121212);

fn fx_startup() -> Effect {
    let timer = EffectTimer::from_ms(3000, QuadIn);

    parallel(&[
        parallel(&[
            sweep_in(Motion::LeftToRight, 100, 20, Color::Black, timer),
            sweep_in(Motion::UpToDown, 100, 20, Color::Black, timer),
        ]),
        prolong_start(500, coalesce((2500, SineOut))),
    ])
}

fn fx_pulsate_selected_tab() -> Effect {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);
    let highlighted_tab = CellFilter::AllOf(vec![
        CellFilter::Layout(layout, 0),
        CellFilter::FgColor(Color::LightYellow),
    ]);

    repeating(hsl_shift_fg([-170.0, 25.0, 30.0], (1000, SineInOut))).with_filter(highlighted_tab)
}

fn fx_change_tab() -> Effect {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]);
    let dissolved = Style::default().fg(Color::White).bg(BG_COLOR);
    let flash_color = Color::from_u32(0x3232030);

    sequence(&[
        with_duration(
            Duration::from_millis(300),
            parallel(&[
                style_all_cells(),
                never_complete(fade_to(flash_color, flash_color, (30, ExpoInOut))),
                never_complete(dissolve_to(dissolved, (125, ExpoInOut))),
                never_complete(fade_to_fg(BG_COLOR, (125, BounceOut))),
            ])
                .with_color_space(ColorSpace::Rgb),
        ),
        parallel(&[
            style_all_cells(),
            fade_from(BG_COLOR, BG_COLOR, (140, Linear)),
            sweep_in(Motion::UpToDown, 40, 0, BG_COLOR, (140, Linear))
                .with_color_space(ColorSpace::Hsl),
        ]),
    ])
        .with_filter(CellFilter::Layout(layout, 1))
}

fn style_all_cells() -> Effect {
    never_complete(effect_fn((), 100_000, |_, _, cells| {
        for (_, cell) in cells {
            if cell.fg == Color::Reset {
                cell.set_fg(Color::White);
            }
            if cell.bg == Color::Reset {
                cell.set_bg(BG_COLOR);
            }
        }
    }))
}

// ── UI ──────────────────────────────────────────────────────────────

fn ui_draw(elapsed: Duration, frame: &mut Frame, app: &mut BeamtermDemo) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());
    let tabs = app
        .tabs
        .titles
        .iter()
        .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::LightGreen))))
        .collect::<Tabs>()
        .block(Block::bordered().title(app.title))
        .highlight_style(Style::default().fg(Color::LightYellow))
        .select(app.tabs.index);
    frame.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_first_tab(frame, app, chunks[1]),
        1 => draw_second_tab(frame, app, chunks[1]),
        2 => draw_third_tab(frame, app, chunks[1]),
        _ => {}
    };
    let area = frame.area();
    app.effects
        .process_effects(elapsed, frame.buffer_mut(), area);
}

fn draw_first_tab(frame: &mut Frame, app: &mut BeamtermDemo, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(9),
        Constraint::Min(8),
        Constraint::Length(7),
    ])
        .split(area);
    draw_gauges(frame, app, chunks[0]);
    draw_charts(frame, app, chunks[1]);
    draw_text(frame, chunks[2]);
}

fn draw_gauges(frame: &mut Frame, app: &mut BeamtermDemo, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
        .margin(1)
        .split(area);
    let block = Block::bordered().title("Graphs");
    frame.render_widget(block, area);

    let label = format!("{:.2}%", app.progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::new().title("Gauge:"))
        .gauge_style(
            Style::default()
                .fg(Color::LightMagenta)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .use_unicode(app.enhanced_graphics)
        .label(label)
        .ratio(app.progress);
    frame.render_widget(gauge, chunks[0]);

    let sparkline = Sparkline::default()
        .block(Block::new().title("Sparkline:"))
        .style(Style::default().fg(Color::LightGreen))
        .data(&app.sparkline.points)
        .bar_set(if app.enhanced_graphics {
            symbols::bar::NINE_LEVELS
        } else {
            symbols::bar::THREE_LEVELS
        });
    frame.render_widget(sparkline, chunks[1]);

    let line_gauge = LineGauge::default()
        .block(Block::new().title("LineGauge:"))
        .filled_style(Style::default().fg(Color::LightMagenta))
        .filled_symbol(if app.enhanced_graphics {
            symbols::line::THICK.horizontal
        } else {
            symbols::line::NORMAL.horizontal
        })
        .ratio(app.progress);
    frame.render_widget(line_gauge, chunks[2]);
}

#[allow(clippy::too_many_lines)]
fn draw_charts(frame: &mut Frame, app: &mut BeamtermDemo, area: Rect) {
    let constraints = if app.show_chart {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let chunks = Layout::horizontal(constraints).split(area);
    {
        let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);
        {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[0]);

            let tasks: Vec<ListItem> = app
                .tasks
                .items
                .iter()
                .map(|i| ListItem::new(vec![text::Line::from(Span::raw(*i))]))
                .collect();
            let tasks = List::new(tasks)
                .block(Block::bordered().title("List"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            frame.render_stateful_widget(tasks, chunks[0], &mut app.tasks.state);

            let info_style = Style::default().fg(Color::Green);
            let warning_style = Style::default().fg(Color::LightYellow);
            let error_style = Style::default().fg(Color::LightMagenta);
            let critical_style = Style::default().fg(Color::LightRed);
            let logs: Vec<ListItem> = app
                .logs
                .items
                .iter()
                .map(|&(evt, level)| {
                    let s = match level {
                        "ERROR" => error_style,
                        "CRITICAL" => critical_style,
                        "WARNING" => warning_style,
                        _ => info_style,
                    };
                    let content = vec![text::Line::from(vec![
                        Span::styled(format!("{level:<9}"), s),
                        Span::raw(evt),
                    ])];
                    ListItem::new(content)
                })
                .collect();
            let logs = List::new(logs).block(Block::bordered().title("List"));
            frame.render_stateful_widget(logs, chunks[1], &mut app.logs.state);
        }

        let barchart = BarChart::default()
            .block(Block::bordered().title("Bar Chart"))
            .data(&app.barchart)
            .bar_width(3)
            .bar_gap(2)
            .bar_set(if app.enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
            .value_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::ITALIC),
            )
            .label_style(Style::default().fg(Color::Yellow))
            .bar_style(Style::default().fg(Color::LightGreen));
        frame.render_widget(barchart, chunks[1]);
    }
    if app.show_chart {
        let x_labels = vec![
            Span::styled(
                format!("{}", app.signals.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{}",
                (app.signals.window[0] + app.signals.window[1]) / 2.0
            )),
            Span::styled(
                format!("{}", app.signals.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::White))
                .data(&app.signals.sin1.points),
            Dataset::default()
                .name("data3")
                .marker(if app.enhanced_graphics {
                    symbols::Marker::Braille
                } else {
                    symbols::Marker::Dot
                })
                .style(Style::default().fg(Color::LightCyan))
                .data(&app.signals.sin2.points),
        ];
        let chart = Chart::new(datasets)
            .block(
                Block::bordered().title(Span::styled(
                    "Chart",
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                )),
            )
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .bounds(app.signals.window)
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([-20.0, 20.0])
                    .labels([
                        Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("0"),
                        Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
            );
        frame.render_widget(chart, chunks[1]);
    }
}

fn draw_text(frame: &mut Frame, area: Rect) {
    let text = vec![
        text::Line::from("This is a paragraph with several lines. You can change style your text the way you want"),
        text::Line::from(""),
        text::Line::from(vec![
            Span::from("For example: "),
            Span::styled("under", Style::default().fg(Color::LightRed)),
            Span::raw(" "),
            Span::styled("the", Style::default().fg(Color::LightGreen)),
            Span::raw(" "),
            Span::styled("rainbow", Style::default().fg(Color::LightCyan)),
            Span::raw("."),
        ]),
        text::Line::from(vec![
            Span::raw("Oh and if you didn't "),
            Span::styled("notice", Style::default().add_modifier(Modifier::ITALIC)),
            Span::raw(" you can "),
            Span::styled("automatically", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled("wrap", Style::default().add_modifier(Modifier::REVERSED)),
            Span::raw(" your "),
            Span::styled("text", Style::default().add_modifier(Modifier::UNDERLINED)),
            Span::raw(".")
        ]),
        text::Line::from(
            "One more thing is that it should display unicode characters: 10\u{20ac}"
        ),
    ];
    let block = Block::bordered().title(Span::styled(
        "Footer",
        Style::default()
            .fg(Color::LightMagenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_second_tab(frame: &mut Frame, app: &mut BeamtermDemo, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).split(area);
    let up_style = Style::default().fg(Color::LightGreen);
    let failure_style = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);
    let rows = app.servers.iter().map(|s| {
        let style = if s.status == "Up" {
            up_style
        } else {
            failure_style
        };
        Row::new(vec![s.name, s.location, s.status]).style(style)
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ],
    )
        .header(
            Row::new(vec!["Server", "Location", "Status"])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1),
        )
        .block(Block::bordered().title("Servers"));
    frame.render_widget(table, chunks[0]);

    let map = Canvas::default()
        .block(Block::bordered().title("World"))
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.layer();
            ctx.draw(&Rectangle {
                x: 0.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
                color: Color::Yellow,
            });
            ctx.draw(&Circle {
                x: app.servers[2].coords.1,
                y: app.servers[2].coords.0,
                radius: 10.0,
                color: Color::LightGreen,
            });
            for (i, s1) in app.servers.iter().enumerate() {
                for s2 in &app.servers[i + 1..] {
                    ctx.draw(&canvas::Line {
                        x1: s1.coords.1,
                        y1: s1.coords.0,
                        y2: s2.coords.0,
                        x2: s2.coords.1,
                        color: Color::Yellow,
                    });
                }
            }
            for server in &app.servers {
                let color = if server.status == "Up" {
                    Color::LightGreen
                } else {
                    Color::Red
                };
                ctx.print(
                    server.coords.1,
                    server.coords.0,
                    Span::styled("X", Style::default().fg(color)),
                );
            }
        })
        .marker(if app.enhanced_graphics {
            symbols::Marker::Braille
        } else {
            symbols::Marker::Dot
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    frame.render_widget(map, chunks[1]);
}

fn draw_third_tab(frame: &mut Frame, _app: &mut BeamtermDemo, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(area);
    let colors = [
        Color::Reset,
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::LightMagenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    let items: Vec<Row> = colors
        .iter()
        .map(|c| {
            let cells = vec![
                ratatui::widgets::Cell::from(Span::raw(format!("{c:?}: "))),
                ratatui::widgets::Cell::from(Span::styled(
                    "Foreground",
                    Style::default().fg(*c),
                )),
                ratatui::widgets::Cell::from(Span::styled(
                    "Background",
                    Style::default().bg(*c),
                )),
            ];
            Row::new(cells)
        })
        .collect();
    let table = Table::new(
        items,
        [
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ],
    )
        .block(Block::bordered().title("Colors"));
    frame.render_widget(table, chunks[0]);
}
