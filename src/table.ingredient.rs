use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

use super::SkeletonTable;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Rows with column separators and optional zebra striping, matching ratatui table layout.";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "elapsed_ms",
        ty: "u64",
        description: "Timestamp driving the animation cycle",
    },
    PropInfo {
        name: "mode",
        ty: "AnimationMode",
        description: "Breathe (uniform pulse), Sweep (traveling highlight), Plasma (dual sine waves), Noise (TV noise)",
    },
    PropInfo {
        name: "rows",
        ty: "u16",
        description: "Number of visible rows (default: 5)",
    },
    PropInfo {
        name: "columns",
        ty: "&[Constraint]",
        description: "Column width constraints",
    },
    PropInfo {
        name: "zebra",
        ty: "bool",
        description: "Alternating row brightness (default: true)",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, braille, name)| -> Box<dyn Ingredient> {
            Box::new(TableVariant {
                epoch: Instant::now(),
                mode,
                braille,
                variant: name,
            })
        })
        .collect()
}

const VARIANTS: &[(AnimationMode, bool, &str)] = &[
    (AnimationMode::Breathe, false, "Breathe (default)"),
    (AnimationMode::Sweep, false, "Sweep"),
    (AnimationMode::Plasma, false, "Plasma"),
    (AnimationMode::Noise, false, "Noise"),
    (AnimationMode::Breathe, true, "Braille Breathe"),
    (AnimationMode::Sweep, true, "Braille Sweep"),
    (AnimationMode::Plasma, true, "Braille Plasma"),
];

const TABLE_COLS: [Constraint; 3] = [
    Constraint::Percentage(30),
    Constraint::Percentage(40),
    Constraint::Percentage(30),
];

const HEADER: [&str; 3] = ["Node", "Region", "Status"];

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct TableVariant {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl Ingredient for TableVariant {
    fn group(&self) -> &str {
        "Table"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::table"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            TableWithHeader {
                ms: elapsed_ms(self.epoch),
                mode: self.mode,
                braille: self.braille,
            },
            None,
            Some(Constraint::Length(6)),
            area,
            buf,
        );
    }
}

/// Composite widget: header row + skeleton table body.
struct TableWithHeader {
    ms: u64,
    mode: AnimationMode,
    braille: bool,
}

impl Widget for TableWithHeader {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, body_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        render_header(&HEADER, &TABLE_COLS, header_area, buf);

        SkeletonTable::new(self.ms)
            .mode(self.mode)
            .braille(self.braille)
            .columns(&TABLE_COLS)
            .rows(5)
            .render(body_area, buf);
    }
}

/// Render column headers matching the constraint layout.
fn render_header(labels: &[&str], cols: &[Constraint], area: Rect, buf: &mut Buffer) {
    let widths: Vec<Constraint> = cols
        .iter()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 {
                vec![Constraint::Length(1), *c]
            } else {
                vec![*c]
            }
        })
        .collect();

    let areas = Layout::horizontal(widths).split(area);

    // Labels land on even indices (0, 2, 4, ...); separators on odd.
    for (i, &label) in labels.iter().enumerate() {
        let idx = if i == 0 { 0 } else { i * 2 };

        if idx < areas.len() {
            Line::from(label)
                .style(Style::new().bold())
                .render(areas[idx], buf);
        }
    }
}
