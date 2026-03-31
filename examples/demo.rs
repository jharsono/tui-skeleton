use std::time::Instant;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table, Widget},
};
use tui_skeleton::{
    AnimationMode, SkeletonBarChart, SkeletonBlock, SkeletonHBarChart, SkeletonLineChart,
    SkeletonStreamingText, SkeletonTable, SkeletonText, TICK_ANIMATED,
};

const BASE: Color = Color::Rgb(45, 45, 50);
const HIGHLIGHT: Color = Color::Rgb(90, 90, 95);

const MODES: [AnimationMode; 4] = [
    AnimationMode::Breathe,
    AnimationMode::Sweep,
    AnimationMode::Plasma,
    AnimationMode::Noise,
];

const MODE_NAMES: [&str; 4] = ["Breathe", "Sweep", "Plasma", "Noise"];

const PHASE_MS: u64 = 5000;
const CYCLE_MS: u64 = PHASE_MS * 2;

struct App {
    epoch: Instant,
    mode_index: usize,
    braille: bool,
    looping: bool,
}

impl App {
    fn new() -> Self {
        Self {
            epoch: Instant::now(),
            mode_index: 0,
            braille: false,
            looping: false,
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

    fn toggle_braille(&mut self) {
        self.braille = !self.braille;
    }

    fn toggle_loop(&mut self) {
        self.looping = !self.looping;
    }

    fn elapsed_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    fn is_skeleton_phase(&self) -> bool {
        !self.looping || (self.elapsed_ms() % CYCLE_MS) < PHASE_MS
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
                    KeyCode::Char('b') => app.toggle_braille(),
                    KeyCode::Char('l') => app.toggle_loop(),
                    _ => {}
                }
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    let ms = app.elapsed_ms();
    let mode = app.mode();
    let braille = app.braille;
    let skeleton = app.is_skeleton_phase();

    let [header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(frame.area());

    let fill_label = if braille { "braille" } else { "solid" };
    let loop_label = if app.looping { "on" } else { "off" };
    let phase_label = if skeleton { "skeleton" } else { "data" };

    let status = format!(
        " tui-skeleton │ mode: {} │ fill: {} │ loop: {} ({}) │ [m] cycle  [b] braille  [l] loop  [q] quit",
        app.mode_name(),
        fill_label,
        loop_label,
        phase_label,
    );
    frame.render_widget(
        Paragraph::new(status).style(Style::new().fg(BORDER_COLOR)),
        header,
    );

    // 3×3 grid for 9 widgets.
    let thirds = [
        Constraint::Percentage(33),
        Constraint::Percentage(34),
        Constraint::Percentage(33),
    ];

    let [row1, row2, row3] = Layout::vertical(thirds).areas(body);
    let [a1, a2, a3] = Layout::horizontal(thirds).areas(row1);
    let [b1, b2, b3] = Layout::horizontal(thirds).areas(row2);
    let [c1, c2, c3] = Layout::horizontal(thirds).areas(row3);

    if skeleton {
        draw_skeleton(frame, ms, mode, braille, a1, a2, a3, b1, b2, b3, c1, c2, c3);
    } else {
        draw_data(frame, a1, a2, a3, b1, b2, b3, c1, c2, c3);
    }
}

// ── Shared constants ──

const TABLE_COLS: [Constraint; 3] = [
    Constraint::Percentage(30),
    Constraint::Percentage(40),
    Constraint::Percentage(30),
];

const TABLE_HEADER: [&str; 3] = ["Node", "Region", "Status"];
const BAR_LABELS: [&str; 6] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const HBAR_LABELS: [&str; 5] = ["Alpha", "Bravo", "Charlie", "Delta", "Echo"];
const HBAR_LABEL_W: u16 = 8;
const KV_KEYS: [&str; 6] = ["Nodes", "Uptime", "Jobs", "Memory", "CPU", "Disk"];
const KV_KEY_W: u16 = 8;

#[expect(clippy::too_many_arguments)]
fn draw_skeleton(
    frame: &mut Frame,
    ms: u64,
    mode: AnimationMode,
    braille: bool,
    a1: Rect,
    a2: Rect,
    a3: Rect,
    b1: Rect,
    b2: Rect,
    b3: Rect,
    c1: Rect,
    c2: Rect,
    c3: Rect,
) {
    let buf = frame.buffer_mut();

    // Block — gauge placeholder (label + bar).
    let block_outer = titled_block("Disk");
    let block_inner = block_outer.inner(a1);
    block_outer.render(a1, buf);

    let [_, label_area, _, bar_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(block_inner);

    Line::from("Disk: ···· / ····")
        .style(Style::new().fg(BORDER_COLOR))
        .render(label_area, buf);

    SkeletonBlock::new(ms)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .render(bar_area, buf);

    // Table — header is known, rows are skeleton.
    let table_block = titled_block("Table");
    let table_inner = table_block.inner(a2);
    table_block.render(a2, buf);
    let [table_header, table_body] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(table_inner);

    for (i, &label) in TABLE_HEADER.iter().enumerate() {
        let col_w = table_header.width / TABLE_HEADER.len() as u16;
        let x = table_header.x + (i as u16) * col_w;

        Line::from(label)
            .style(Style::new().bold())
            .render(Rect::new(x, table_header.y, col_w, 1), buf);
    }

    SkeletonTable::new(ms)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .columns(&TABLE_COLS)
        .rows(8)
        .render(table_body, buf);

    // Bar Chart — x-axis labels are known, bars are skeleton.
    let bar_block = titled_block("Bar Chart");
    let bar_inner = bar_block.inner(a3);
    bar_block.render(a3, buf);
    let [bar_body, bar_labels] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(bar_inner);

    render_bar_labels(bar_labels, buf);

    SkeletonBarChart::new(ms)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .bars(6)
        .bar_width(3)
        .render(bar_body, buf);

    // List — no gaps between items, matching data phase.
    SkeletonText::new(ms)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .line_widths(&[0.45, 0.35, 0.55, 0.30, 0.40])
        .block(titled_block("List"))
        .render(b1, buf);

    // Text — no known chrome.
    SkeletonText::new(ms)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .block(titled_block("Text"))
        .render(b2, buf);

    // H-Bar Chart — row labels are known, bars are skeleton.
    let hbar_block = titled_block("H-Bar Chart");
    let hbar_inner = hbar_block.inner(b3);
    hbar_block.render(b3, buf);

    render_hbar_labels(hbar_inner, buf);

    let hbar_x = hbar_inner.x + HBAR_LABEL_W;
    let hbar_w = hbar_inner.width.saturating_sub(HBAR_LABEL_W);

    if hbar_w > 0 {
        SkeletonHBarChart::new(ms)
            .mode(mode)
            .braille(braille)
            .base(BASE)
            .highlight(HIGHLIGHT)
            .bars(5)
            .render(
                Rect::new(hbar_x, hbar_inner.y, hbar_w, hbar_inner.height),
                buf,
            );
    }

    // Line Chart — no known chrome.
    SkeletonLineChart::new(ms)
        .drift_ms(0)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .lines(2)
        .block(titled_block("Line Chart"))
        .render(c1, buf);

    // KV Table — keys are known, values are skeleton.
    let kv_block = titled_block("KV Table");
    let kv_inner = kv_block.inner(c2);
    kv_block.render(c2, buf);

    render_kv_skeleton(kv_inner, buf, ms, mode, braille);

    // Streaming Text — no known chrome.
    SkeletonStreamingText::new(ms)
        .mode(mode)
        .braille(braille)
        .base(BASE)
        .highlight(HIGHLIGHT)
        .repeat(true)
        .block(titled_block("Streaming Text"))
        .render(c3, buf);
}

#[expect(clippy::too_many_arguments)]
fn draw_data(
    frame: &mut Frame,
    a1: Rect,
    a2: Rect,
    a3: Rect,
    b1: Rect,
    b2: Rect,
    b3: Rect,
    c1: Rect,
    c2: Rect,
    c3: Rect,
) {
    // Block — storage gauge.
    let block = titled_block("Disk");
    let block_inner = block.inner(a1);
    frame.render_widget(block, a1);

    let [_, label_area, _, bar_area, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(block_inner);

    Line::from("Disk: 750 GB / 1 TB").render(label_area, frame.buffer_mut());

    let used_frac = 0.75;
    let used_w = ((bar_area.width as f32) * used_frac) as u16;
    let used_color = Color::Rgb(99, 102, 241);
    let free_color = Color::Rgb(60, 60, 60);

    for x in 0..bar_area.width {
        let color = if x < used_w { used_color } else { free_color };

        frame.buffer_mut()[(bar_area.x + x, bar_area.y)]
            .set_char('█')
            .set_style(Style::default().fg(color));
    }

    // Table
    let table_cols = [
        Constraint::Percentage(30),
        Constraint::Percentage(40),
        Constraint::Percentage(30),
    ];
    let header = Row::new(["Node", "Region", "Status"]).style(Style::new().bold());
    let rows = [
        Row::new(["use1a", "us-east-1a", "Online"]),
        Row::new(["usw2b", "us-west-2b", "Online"]),
        Row::new(["euc1", "eu-central-1", "Degraded"]),
        Row::new(["aps1", "ap-south-1", "Online"]),
        Row::new(["euw1", "eu-west-1", "Offline"]),
    ];
    frame.render_widget(
        Table::new(rows, table_cols)
            .header(header)
            .block(titled_block("Table")),
        a2,
    );

    // Bar Chart
    let bar_block = titled_block("Bar Chart");
    let bar_inner = bar_block.inner(a3);
    frame.render_widget(bar_block, a3);
    render_vertical_bars(bar_inner, frame.buffer_mut());

    // List
    let items: Vec<Line> = [
        "Eggs",
        "Milk",
        "Bread",
        "Butter",
        "Cheese",
        "Apples",
        "Bananas",
        "Carrots",
        "Onions",
        "Garlic",
        "Olive oil",
        "Rice",
        "Pasta",
        "Tomatoes",
        "Spinach",
    ]
    .into_iter()
    .map(Line::from)
    .collect();
    frame.render_widget(Paragraph::new(items).block(titled_block("List")), b1);

    // Text
    frame.render_widget(
        Paragraph::new(
            "Lorem ipsum dolor sit amet,\nconsectetur adipiscing elit,\nsed do eiusmod tempor\nincididunt ut labore et dolore\nmagna aliqua. Ut enim ad minim\nveniam, quis nostrud exerci-\ntation ullamco laboris nisi ut\naliquip ex ea commodo consequat.\nDuis aute irure dolor in repre-\nhenderit in voluptate velit\nesse cillum dolore eu fugiat\nnulla pariatur. Excepteur sint\noccaecat cupidatat non proident.",
        )
        .block(titled_block("Text")),
        b2,
    );

    // H-Bar Chart
    let hbar_block = titled_block("H-Bar Chart");
    let hbar_inner = hbar_block.inner(b3);
    frame.render_widget(hbar_block, b3);
    render_horizontal_bars(hbar_inner, frame.buffer_mut());

    // Line Chart — frozen wave from the actual widget with data colors.
    let lc_block = titled_block("Line Chart");
    let lc_inner = lc_block.inner(c1);
    frame.render_widget(lc_block, c1);
    SkeletonLineChart::new(0)
        .lines(2)
        .base(Color::Rgb(74, 222, 128))
        .highlight(Color::Rgb(120, 240, 160))
        .render(lc_inner, frame.buffer_mut());

    // KV Table — spaced to match skeleton stride.
    let kv_block = titled_block("KV Table");
    let kv_inner = kv_block.inner(c2);
    frame.render_widget(kv_block, c2);
    render_kv_data(kv_inner, frame.buffer_mut());

    // Streaming Text
    frame.render_widget(
        Paragraph::new(
            "Lorem ipsum dolor sit amet, consectetur\nadipiscing elit, sed do eiusmod tempor\nincididunt ut labore et dolore magna\naliqua. Ut enim ad minim veniam, quis\nnostrud exercitation ullamco laboris\nnisi ut aliquip ex ea commodo consequat.",
        )
        .block(titled_block("Streaming Text")),
        c3,
    );
}

// ── Chrome helpers (shared by skeleton and data phases) ──

fn render_bar_labels(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let bar_width = 3u16;
    let stride = bar_width + 1;

    for (i, &label) in BAR_LABELS.iter().enumerate() {
        let x = area.x + (i as u16) * stride;

        if x + bar_width > area.right() {
            break;
        }

        Line::from(label).render(Rect::new(x, area.y, bar_width, 1), buf);
    }
}

fn render_hbar_labels(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    for (i, &label) in HBAR_LABELS.iter().enumerate() {
        let y = area.y + (i as u16) * 2;

        if y >= area.bottom() {
            break;
        }

        Line::from(label)
            .style(Style::new().bold())
            .render(Rect::new(area.x, y, HBAR_LABEL_W, 1), buf);
    }
}

fn render_kv_skeleton(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    ms: u64,
    mode: AnimationMode,
    braille: bool,
) {
    let value_start = KV_KEY_W + 2;
    let value_widths: [f32; 6] = [0.25, 0.55, 0.20, 0.45, 0.35, 0.30];

    for (i, &key) in KV_KEYS.iter().enumerate() {
        let y = area.y + (i as u16) * 2;

        if y >= area.bottom() {
            break;
        }

        Line::from(format!("{key}:"))
            .style(Style::new().bold())
            .render(Rect::new(area.x, y, KV_KEY_W, 1), buf);

        let val_x = area.x + value_start;
        let val_w = area.width.saturating_sub(value_start);

        if val_w > 0 {
            SkeletonHBarChart::new(ms)
                .mode(mode)
                .braille(braille)
                .base(BASE)
                .highlight(HIGHLIGHT)
                .bars(1)
                .bar_height(1)
                .widths(&[value_widths[i % value_widths.len()]])
                .render(Rect::new(val_x, y, val_w, 1), buf);
        }
    }
}

// ── Data-phase helpers ──

fn render_kv_data(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let values = ["6", "14d 3h", "25", "18.4 GB", "34%", "128 GB"];

    for (i, (&key, &val)) in KV_KEYS.iter().zip(&values).enumerate() {
        let y = area.y + (i as u16) * 2;

        if y >= area.bottom() {
            break;
        }

        Line::from(format!("{key}:"))
            .style(Style::new().bold())
            .render(Rect::new(area.x, y, KV_KEY_W, 1), buf);

        let val_x = area.x + KV_KEY_W + 2;
        let val_w = area.width.saturating_sub(KV_KEY_W + 2);

        if val_w > 0 {
            Line::from(val).render(Rect::new(val_x, y, val_w, 1), buf);
        }
    }
}

fn render_vertical_bars(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let values: [u16; 6] = [60, 85, 45, 95, 70, 55];
    let max = 95u16;
    let bar_width = 3u16;
    let stride = bar_width + 1;
    let bar_color = Color::Rgb(74, 222, 128);

    let [bars_area, label_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

    for (i, (&label, &val)) in BAR_LABELS.iter().zip(&values).enumerate() {
        let x = bars_area.x + (i as u16) * stride;

        if x + bar_width > bars_area.right() {
            break;
        }

        let bar_h = ((bars_area.height as u32) * val as u32 / max as u32) as u16;
        let bar_top = bars_area.y + bars_area.height - bar_h;

        for dy in 0..bar_h {
            for dx in 0..bar_width {
                buf[(x + dx, bar_top + dy)]
                    .set_char('█')
                    .set_style(Style::default().fg(bar_color));
            }
        }

        Line::from(label).render(Rect::new(x, label_area.y, bar_width, 1), buf);
    }
}

fn render_horizontal_bars(area: Rect, buf: &mut ratatui::buffer::Buffer) {
    let fracs: [f32; 5] = [0.85, 0.60, 0.95, 0.45, 0.75];
    let bar_color = Color::Rgb(99, 102, 241);

    for (i, (&label, &frac)) in HBAR_LABELS.iter().zip(&fracs).enumerate() {
        let y = area.y + (i as u16) * 2;

        if y >= area.bottom() {
            break;
        }

        Line::from(label)
            .style(Style::new().bold())
            .render(Rect::new(area.x, y, HBAR_LABEL_W, 1), buf);

        let bar_x = area.x + HBAR_LABEL_W;
        let bar_max = area.width.saturating_sub(HBAR_LABEL_W);
        let bar_w = ((bar_max as f32) * frac) as u16;

        for dx in 0..bar_w.min(bar_max) {
            buf[(bar_x + dx, y)]
                .set_char('█')
                .set_style(Style::default().fg(bar_color));
        }
    }
}

const BORDER_COLOR: Color = Color::Rgb(255, 255, 255);

fn titled_block(title: &str) -> Block<'_> {
    Block::default()
        .title(Line::from(format!(" {title} ")).style(Style::new().fg(BORDER_COLOR)))
        .borders(Borders::ALL)
        .border_set(ratatui::symbols::border::ROUNDED)
        .border_style(Style::new().fg(BORDER_COLOR))
}
