use std::time::Instant;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Row, Table, Widget};
use ratatui_core::buffer::Buffer;
use tui_pantry::Ingredient;

use crate::{
    AnimationMode, SkeletonBarChart, SkeletonBrailleBar, SkeletonHBarChart, SkeletonList,
    SkeletonTable, SkeletonText,
};

/// Full cycle: 5s skeleton, 5s content, repeat.
const PHASE_MS: u64 = 5000;
const CYCLE_MS: u64 = PHASE_MS * 2;

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    let modes: &[(AnimationMode, bool, &str)] = &[
        (AnimationMode::Breathe, false, "Breathe (default)"),
        (AnimationMode::Sweep, false, "Sweep"),
        (AnimationMode::Plasma, false, "Plasma"),
        (AnimationMode::Noise, false, "Noise"),
        (AnimationMode::Breathe, true, "Braille Breathe"),
        (AnimationMode::Sweep, true, "Braille Sweep"),
        (AnimationMode::Plasma, true, "Braille Plasma"),
    ];

    let mut out: Vec<Box<dyn Ingredient>> = Vec::new();

    for &(mode, braille, name) in modes {
        out.push(Box::new(LoadingTable::new(mode, braille, name)));
        out.push(Box::new(LoadingArticle::new(mode, braille, name)));
        out.push(Box::new(LoadingSidebar::new(mode, braille, name)));
        out.push(Box::new(LoadingDashboard::new(mode, braille, name)));
        out.push(Box::new(LoadingGauges::new(mode, braille, name)));
    }

    out
}

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

fn is_skeleton_phase(elapsed: u64) -> bool {
    (elapsed % CYCLE_MS) < PHASE_MS
}

// ── Loading Table ──
//
// Header row is always visible — only row data is unknown.

const TABLE_COLS: [Constraint; 4] = [
    Constraint::Percentage(25),
    Constraint::Percentage(25),
    Constraint::Percentage(25),
    Constraint::Percentage(25),
];

const TABLE_HEADER: [&str; 4] = ["Node", "Region", "CPU", "Status"];

struct LoadingTable {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl LoadingTable {
    fn new(mode: AnimationMode, braille: bool, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
            braille,
            variant,
        }
    }
}

impl Ingredient for LoadingTable {
    fn tab(&self) -> &str {
        "Panes"
    }
    fn group(&self) -> &str {
        "Data Table"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::use_cases"
    }
    fn description(&self) -> &str {
        "Header always visible; row data replaced by skeleton during loading."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);
        let header = Row::new(TABLE_HEADER).style(Style::new().bold());

        if is_skeleton_phase(ms) {
            // Header is known; skeleton fills the data rows below it.
            let [header_area, body_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

            Table::new(std::iter::empty::<Row>(), TABLE_COLS)
                .header(header)
                .render(header_area, buf);

            SkeletonTable::new(ms)
                .mode(self.mode)
                .braille(self.braille)
                .columns(&TABLE_COLS)
                .rows(5)
                .render(body_area, buf);
        } else {
            let rows = [
                Row::new(["use1a", "us-east-1a", "34%", "Online"]),
                Row::new(["usw2b", "us-west-2b", "61%", "Online"]),
                Row::new(["euc1", "eu-central-1", "88%", "Degraded"]),
                Row::new(["aps1", "ap-south-1", "22%", "Online"]),
                Row::new(["euw1", "eu-west-1", "—", "Offline"]),
            ];

            Table::new(rows, TABLE_COLS)
                .header(header)
                .render(area, buf);
        }
    }
}

// ── Loading Article ──
//
// Title is always visible — body text is unknown.

const ARTICLE_WIDTHS: &[f32] = &[1.0, 1.0, 0.85, 1.0, 0.6];

struct LoadingArticle {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl LoadingArticle {
    fn new(mode: AnimationMode, braille: bool, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
            braille,
            variant,
        }
    }
}

impl Ingredient for LoadingArticle {
    fn tab(&self) -> &str {
        "Panes"
    }
    fn group(&self) -> &str {
        "Article"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::use_cases"
    }
    fn description(&self) -> &str {
        "Title always visible; body text replaced by skeleton during loading."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);
        let constrained = Rect::new(area.x, area.y, area.width.min(52), area.height.min(7));

        let [title_area, _, body_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(constrained);

        Line::from("Lorem Ipsum")
            .style(Style::new().bold())
            .render(title_area, buf);

        if is_skeleton_phase(ms) {
            SkeletonText::new(ms)
                .mode(self.mode)
                .braille(self.braille)
                .line_widths(ARTICLE_WIDTHS)
                .render(body_area, buf);
        } else {
            Paragraph::new(vec![
                Line::from("Dolor sit amet, consectetur adipiscing elit, sed"),
                Line::from("do eiusmod tempor incididunt ut labore et dolore"),
                Line::from("magna aliqua. Ut enim ad minim veniam,"),
                Line::from("quis nostrud exercitation ullamco laboris nisi"),
                Line::from("ut aliquip ex ea commodo."),
            ])
            .render(body_area, buf);
        }
    }
}

// ── Loading Sidebar ──
//
// Section header is always visible — menu items are the data.

struct LoadingSidebar {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl LoadingSidebar {
    fn new(mode: AnimationMode, braille: bool, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
            braille,
            variant,
        }
    }
}

impl Ingredient for LoadingSidebar {
    fn tab(&self) -> &str {
        "Panes"
    }
    fn group(&self) -> &str {
        "Sidebar"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::use_cases"
    }
    fn description(&self) -> &str {
        "Section header always visible; menu items replaced by skeleton during loading."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);

        let [header_area, _, body_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);

        Line::from("Navigation")
            .style(Style::new().bold())
            .render(header_area, buf);

        if is_skeleton_phase(ms) {
            SkeletonList::new(ms)
                .mode(self.mode)
                .braille(self.braille)
                .items(5)
                .render(body_area, buf);
        } else {
            let items = ["Dashboard", "Network", "Models", "Logs", "Settings"];

            for (i, item) in items.iter().enumerate() {
                let y = body_area.y + (i as u16) * 2;

                if y >= body_area.bottom() {
                    break;
                }

                Line::from(*item).render(Rect::new(body_area.x, y, body_area.width, 1), buf);
            }
        }
    }
}

// ── Loading Dashboard (composite) ──
//
// Layout structure, bar labels, KV keys, and table header are always
// visible. Chart bars, KV values, and table rows are unknown.

const DASHBOARD_TABLE_COLS: [Constraint; 3] = [
    Constraint::Percentage(30),
    Constraint::Percentage(40),
    Constraint::Percentage(30),
];

const DASHBOARD_HEADER: [&str; 3] = ["Node", "Region", "Status"];

const BAR_LABELS: [&str; 6] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const KV_KEYS: [&str; 4] = ["Nodes", "Uptime", "Jobs", "Memory"];

struct LoadingDashboard {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl LoadingDashboard {
    fn new(mode: AnimationMode, braille: bool, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
            braille,
            variant,
        }
    }
}

impl Ingredient for LoadingDashboard {
    fn tab(&self) -> &str {
        "Panes"
    }
    fn group(&self) -> &str {
        "Dashboard"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::use_cases"
    }
    fn description(&self) -> &str {
        "Layout chrome always visible; data regions replaced by skeletons during loading."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);

        let [top, bottom] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

        let [chart_area, kv_area] =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(top);

        let [table_area] = Layout::horizontal([Constraint::Percentage(100)]).areas(bottom);

        if is_skeleton_phase(ms) {
            // Bar chart: labels on x-axis are known, bars are skeleton.
            let [bars_area, labels_area] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(chart_area);

            SkeletonBarChart::new(ms)
                .mode(self.mode)
                .braille(self.braille)
                .render(bars_area, buf);

            render_bar_labels(labels_area, buf);

            // KV table: keys are known, values are skeleton.
            render_kv_skeleton(kv_area, buf, ms, self.mode, self.braille);

            // Table: header is known, rows are skeleton.
            let [header_area, body_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(table_area);

            Table::new(std::iter::empty::<Row>(), DASHBOARD_TABLE_COLS)
                .header(Row::new(DASHBOARD_HEADER).style(Style::new().bold()))
                .render(header_area, buf);

            SkeletonTable::new(ms)
                .mode(self.mode)
                .braille(self.braille)
                .columns(&DASHBOARD_TABLE_COLS)
                .rows(4)
                .render(body_area, buf);
        } else {
            render_simple_bars(chart_area, buf);
            render_simple_kv(kv_area, buf);

            let header = Row::new(DASHBOARD_HEADER).style(Style::new().bold());
            let rows = [
                Row::new(["use1a", "us-east-1a", "Online"]),
                Row::new(["usw2b", "us-west-2b", "Online"]),
                Row::new(["euc1", "eu-central-1", "Degraded"]),
                Row::new(["euw1", "eu-west-1", "Offline"]),
            ];

            Table::new(rows, DASHBOARD_TABLE_COLS)
                .header(header)
                .render(table_area, buf);
        }
    }
}

// ── Loading Gauges (braille bars) ──
//
// Labels are always visible — only the bar data is unknown.

const GAUGE_LABELS: [&str; 4] = ["CPU", "Memory", "Disk", "Network"];
const GAUGE_VALUES: [f32; 4] = [0.62, 0.84, 0.38, 0.51];
const GAUGE_LABEL_WIDTH: u16 = 8;

struct LoadingGauges {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl LoadingGauges {
    fn new(mode: AnimationMode, braille: bool, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
            braille,
            variant,
        }
    }
}

impl Ingredient for LoadingGauges {
    fn tab(&self) -> &str {
        "Panes"
    }
    fn group(&self) -> &str {
        "Gauges"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::use_cases"
    }
    fn description(&self) -> &str {
        "Labels always visible; gauge bars replaced by skeleton during loading."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);

        for (i, &label) in GAUGE_LABELS.iter().enumerate() {
            let y = area.y + (i as u16) * 2;

            if y >= area.bottom() {
                break;
            }

            // Label is always visible.
            Line::from(format!("{label:<8}"))
                .style(Style::new().bold())
                .render(
                    Rect::new(area.x, y, GAUGE_LABEL_WIDTH.min(area.width), 1),
                    buf,
                );

            let bar_x = area.x + GAUGE_LABEL_WIDTH;
            let bar_width = area.width.saturating_sub(GAUGE_LABEL_WIDTH);

            if bar_width == 0 {
                continue;
            }

            let bar_area = Rect::new(bar_x, y, bar_width, 1);

            if is_skeleton_phase(ms) {
                let use_braille = self.braille || self.mode == AnimationMode::Noise;

                if use_braille {
                    SkeletonBrailleBar::new(ms)
                        .mode(self.mode)
                        .bars(1)
                        .fills(&[1.0])
                        .render(bar_area, buf);
                } else {
                    SkeletonHBarChart::new(ms)
                        .mode(self.mode)
                        .bars(1)
                        .bar_height(1)
                        .widths(&[1.0])
                        .render(bar_area, buf);
                }
            } else {
                render_braille_gauge(bar_area, buf, GAUGE_VALUES[i]);
            }
        }
    }
}

// ── Helpers ──

fn render_bar_labels(area: Rect, buf: &mut Buffer) {
    let bar_width = 3u16;
    let stride = bar_width + 1;

    for (i, label) in BAR_LABELS.iter().enumerate() {
        let x = area.x + (i as u16) * stride;

        if x + bar_width > area.right() {
            break;
        }

        for (ci, ch) in label.chars().take(bar_width as usize).enumerate() {
            buf[(x + ci as u16, area.y)].set_char(ch);
        }
    }
}

/// Varying value widths so KV skeletons look like different-length text.
const KV_VALUE_WIDTHS: [f32; 4] = [0.25, 0.55, 0.20, 0.45];

fn render_kv_skeleton(area: Rect, buf: &mut Buffer, ms: u64, mode: AnimationMode, braille: bool) {
    let key_width = 8u16;
    let value_start = key_width + 2;

    for (i, &key) in KV_KEYS.iter().enumerate() {
        let y = area.y + (i as u16) * 2;

        if y >= area.bottom() {
            break;
        }

        // Key is always visible.
        Line::from(format!("{key}:"))
            .style(Style::new().bold())
            .render(Rect::new(area.x, y, key_width, 1), buf);

        // Value is skeleton with varying width.
        let val_x = area.x + value_start;
        let val_width = area.width.saturating_sub(value_start);

        if val_width > 0 {
            let frac = KV_VALUE_WIDTHS[i % KV_VALUE_WIDTHS.len()];

            SkeletonHBarChart::new(ms)
                .mode(mode)
                .braille(braille)
                .bars(1)
                .bar_height(1)
                .widths(&[frac])
                .render(Rect::new(val_x, y, val_width, 1), buf);
        }
    }
}

fn render_simple_bars(area: Rect, buf: &mut Buffer) {
    let values = [60u16, 85, 45, 95, 70, 55];
    let max = values.iter().copied().max().unwrap_or(1);
    let bar_width = 3u16;
    let stride = bar_width + 1;

    let [bars_area, labels_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

    for (i, (label, &val)) in BAR_LABELS.iter().zip(&values).enumerate() {
        let x = bars_area.x + (i as u16) * stride;

        if x + bar_width > bars_area.right() {
            break;
        }

        let bar_height = ((bars_area.height as u32) * val as u32 / max as u32) as u16;
        let bar_top = bars_area.y + bars_area.height - bar_height;

        for dy in 0..bar_height {
            for dx in 0..bar_width {
                buf[(x + dx, bar_top + dy)]
                    .set_char('█')
                    .set_style(Style::default().fg(Color::Rgb(74, 222, 128)));
            }
        }

        for (ci, ch) in label.chars().take(bar_width as usize).enumerate() {
            buf[(x + ci as u16, labels_area.y)].set_char(ch);
        }
    }
}

fn render_simple_kv(area: Rect, buf: &mut Buffer) {
    let pairs = [
        ("Nodes", "6"),
        ("Uptime", "14d 3h"),
        ("Jobs", "25"),
        ("Memory", "18.4 GB"),
    ];

    for (i, (key, val)) in pairs.iter().enumerate() {
        let y = area.y + (i as u16) * 2;

        if y >= area.bottom() {
            break;
        }

        let key_line = Line::from(format!("{key}:")).style(Style::new().bold());
        key_line.render(Rect::new(area.x, y, area.width, 1), buf);

        let val_x = area.x + key.len() as u16 + 2;

        if val_x < area.right() {
            Line::from(*val).render(Rect::new(val_x, y, area.right() - val_x, 1), buf);
        }
    }
}

fn render_braille_gauge(area: Rect, buf: &mut Buffer, frac: f32) {
    let fill_color = Color::Rgb(99, 102, 241);
    let empty_color = Color::Rgb(60, 60, 60);
    let filled = ((frac * area.width as f32) as u16).min(area.width);

    for col in 0..area.width {
        let x = area.x + col;
        let glyph = match col {
            0 => "\u{28BE}",
            c if c == area.width - 1 => "\u{2877}",
            _ => "\u{28FF}",
        };
        let fg = if col < filled {
            fill_color
        } else {
            empty_color
        };

        buf[(x, area.y)]
            .set_symbol(glyph)
            .set_style(Style::default().fg(fg));
    }
}
