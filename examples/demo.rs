use std::time::Instant;

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use tui_skeleton::{
    AnimationMode, SkeletonBarChart, SkeletonBlock, SkeletonHBarChart, SkeletonKvTable,
    SkeletonLineChart, SkeletonList, SkeletonStreamingText, SkeletonTable, SkeletonText,
    TICK_ANIMATED,
};

const BASE: Color = Color::Rgb(30, 22, 58);
const HIGHLIGHT: Color = Color::Rgb(49, 40, 78);

const MODES: [AnimationMode; 3] = [
    AnimationMode::Breathe,
    AnimationMode::Sweep,
    AnimationMode::Plasma,
];

const MODE_NAMES: [&str; 3] = ["Breathe", "Sweep", "Plasma"];

struct App {
    epoch: Instant,
    mode_index: usize,
}

impl App {
    fn new() -> Self {
        Self {
            epoch: Instant::now(),
            mode_index: 0,
        }
    }

    fn mode(&self) -> AnimationMode {
        MODES[self.mode_index]
    }

    fn mode_name(&self) -> &str {
        MODE_NAMES[self.mode_index]
    }

    fn cycle_mode(&mut self) {
        self.mode_index = (self.mode_index + 1) % MODES.len();
    }

    fn elapsed_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(TICK_ANIMATED)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('m') | KeyCode::Tab => app.cycle_mode(),
                    _ => {}
                }
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    let ms = app.elapsed_ms();
    let mode = app.mode();

    let [header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    let status = format!(
        " tui-skeleton │ mode: {} │ [m] cycle  [q] quit",
        app.mode_name()
    );
    frame.render_widget(
        Paragraph::new(status).style(Style::new().fg(Color::DarkGray)),
        header,
    );

    // 3×3 grid for all 9 widgets.
    let thirds = [
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ];

    let [row_1, row_2, row_3] = Layout::vertical(thirds).areas(body);
    let [a1, b1, c1] = Layout::horizontal(thirds).areas(row_1);
    let [a2, b2, c2] = Layout::horizontal(thirds).areas(row_2);
    let [a3, b3, c3] = Layout::horizontal(thirds).areas(row_3);

    let table_cols = [
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
    ];

    frame.render_widget(
        SkeletonBlock::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .block(titled_block("Block")),
        a1,
    );

    frame.render_widget(
        SkeletonTable::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .columns(&table_cols)
            .rows(8)
            .block(titled_block("Table")),
        b1,
    );

    frame.render_widget(
        SkeletonBarChart::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .bars(6)
            .bar_width(2)
            .block(titled_block("Bar Chart")),
        c1,
    );

    frame.render_widget(
        SkeletonList::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .items(6)
            .block(titled_block("List")),
        a2,
    );

    frame.render_widget(
        SkeletonText::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .block(titled_block("Text")),
        b2,
    );

    frame.render_widget(
        SkeletonHBarChart::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .bars(6)
            .bar_height(1)
            .block(titled_block("H-Bar Chart")),
        c2,
    );

    frame.render_widget(
        SkeletonLineChart::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .lines(2)
            .block(titled_block("Line Chart")),
        a3,
    );

    frame.render_widget(
        SkeletonKvTable::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .pairs(6)
            .key_width(8)
            .block(titled_block("KV Table")),
        b3,
    );

    frame.render_widget(
        SkeletonStreamingText::new(ms)
            .mode(mode)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .repeat(true)
            .block(titled_block("Streaming Text")),
        c3,
    );
}

fn titled_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::DarkGray))
}
