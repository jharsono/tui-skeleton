use std::time::Instant;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Row, Table, Widget};
use ratatui_core::buffer::Buffer;
use tui_pantry::Ingredient;

use crate::{
    AnimationMode, SkeletonBarChart, SkeletonKvTable, SkeletonList, SkeletonTable, SkeletonText,
};

/// Full cycle: 5s skeleton, 5s content, repeat.
const PHASE_MS: u64 = 5000;
const CYCLE_MS: u64 = PHASE_MS * 2;

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    let modes = [
        (AnimationMode::Breathe, "Breathe (default)"),
        (AnimationMode::Sweep, "Sweep"),
        (AnimationMode::Plasma, "Plasma"),
    ];

    let mut out: Vec<Box<dyn Ingredient>> = Vec::new();

    for &(mode, name) in &modes {
        out.push(Box::new(LoadingTable::new(mode, name)));
        out.push(Box::new(LoadingArticle::new(mode, name)));
        out.push(Box::new(LoadingSidebar::new(mode, name)));
        out.push(Box::new(LoadingDashboard::new(mode, name)));
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

struct LoadingTable {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl LoadingTable {
    fn new(mode: AnimationMode, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
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
        "Cycles between SkeletonTable and populated data table every 5s."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);
        let cols = [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ];

        if is_skeleton_phase(ms) {
            SkeletonTable::new(ms)
                .mode(self.mode)
                .columns(&cols)
                .rows(5)
                .render(area, buf);
        } else {
            let header = Row::new(["Node", "Region", "CPU", "Status"]).style(Style::new().bold());
            let rows = [
                Row::new(["use1a", "us-east-1a", "34%", "Online"]),
                Row::new(["usw2b", "us-west-2b", "61%", "Online"]),
                Row::new(["euc1", "eu-central-1", "88%", "Degraded"]),
                Row::new(["aps1", "ap-south-1", "22%", "Online"]),
                Row::new(["euw1", "eu-west-1", "—", "Offline"]),
            ];

            Table::new(rows, cols).header(header).render(area, buf);
        }
    }
}

// ── Loading Article ──

const ARTICLE_WIDTHS: &[f32] = &[1.0, 1.0, 0.85, 1.0, 0.6];

struct LoadingArticle {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl LoadingArticle {
    fn new(mode: AnimationMode, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
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
        "Cycles between SkeletonText and rendered paragraph every 5s."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);
        let constrained = Rect::new(area.x, area.y, area.width.min(52), area.height.min(5));

        if is_skeleton_phase(ms) {
            SkeletonText::new(ms)
                .mode(self.mode)
                .line_widths(ARTICLE_WIDTHS)
                .render(constrained, buf);
        } else {
            Paragraph::new(vec![
                Line::from("Lorem ipsum dolor sit amet, consectetur adipiscing"),
                Line::from("elit, sed do eiusmod tempor incididunt ut labore et"),
                Line::from("dolore magna aliqua. Ut enim ad minim"),
                Line::from("veniam, quis nostrud exercitation ullamco laboris"),
                Line::from("nisi ut aliquip ex ea commodo."),
            ])
            .render(constrained, buf);
        }
    }
}

// ── Loading Sidebar ──

struct LoadingSidebar {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl LoadingSidebar {
    fn new(mode: AnimationMode, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
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
        "Cycles between SkeletonList and populated menu every 5s."
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let ms = elapsed_ms(self.epoch);

        if is_skeleton_phase(ms) {
            SkeletonList::new(ms)
                .mode(self.mode)
                .items(5)
                .render(area, buf);
        } else {
            let items = ["Dashboard", "Network", "Models", "Logs", "Settings"];

            for (i, item) in items.iter().enumerate() {
                let y = area.y + (i as u16) * 2;

                if y >= area.bottom() {
                    break;
                }

                Line::from(*item).render(Rect::new(area.x, y, area.width, 1), buf);
            }
        }
    }
}

// ── Loading Dashboard (composite) ──

struct LoadingDashboard {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl LoadingDashboard {
    fn new(mode: AnimationMode, variant: &'static str) -> Self {
        Self {
            epoch: Instant::now(),
            mode,
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
        "Multi-panel skeleton layout cycling with populated dashboard every 5s."
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

        let cols = [
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ];

        if is_skeleton_phase(ms) {
            SkeletonBarChart::new(ms)
                .mode(self.mode)
                .render(chart_area, buf);

            SkeletonKvTable::new(ms)
                .mode(self.mode)
                .pairs(4)
                .render(kv_area, buf);

            SkeletonTable::new(ms)
                .mode(self.mode)
                .columns(&cols)
                .rows(4)
                .render(table_area, buf);
        } else {
            render_simple_bars(chart_area, buf);
            render_simple_kv(kv_area, buf);

            let header = Row::new(["Node", "Region", "Status"]).style(Style::new().bold());
            let rows = [
                Row::new(["use1a", "us-east-1a", "Online"]),
                Row::new(["usw2b", "us-west-2b", "Online"]),
                Row::new(["euc1", "eu-central-1", "Degraded"]),
                Row::new(["euw1", "eu-west-1", "Offline"]),
            ];

            Table::new(rows, cols)
                .header(header)
                .render(table_area, buf);
        }
    }
}

// ── Helpers for "loaded" content ──

fn render_simple_bars(area: Rect, buf: &mut Buffer) {
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let values = [60u16, 85, 45, 95, 70, 55];
    let max = values.iter().copied().max().unwrap_or(1);
    let bar_width = 3u16;
    let stride = bar_width + 1;

    for (i, (label, &val)) in labels.iter().zip(&values).enumerate() {
        let x = area.x + (i as u16) * stride;

        if x + bar_width > area.right() {
            break;
        }

        let bar_height = ((area.height.saturating_sub(1) as u32) * val as u32 / max as u32) as u16;
        let bar_top = area.y + area.height - 1 - bar_height;

        for dy in 0..bar_height {
            for dx in 0..bar_width {
                buf[(x + dx, bar_top + dy)]
                    .set_char('█')
                    .set_style(Style::default().fg(Color::Rgb(74, 222, 128)));
            }
        }

        let label_y = area.y + area.height - 1;

        for (ci, ch) in label.chars().take(bar_width as usize).enumerate() {
            buf[(x + ci as u16, label_y)].set_char(ch);
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
