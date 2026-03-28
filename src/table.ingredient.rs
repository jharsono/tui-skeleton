use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{layout::render_centered, Ingredient, PropInfo};

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
        description:
            "Breathe (uniform pulse), Sweep (traveling highlight), Plasma (dual sine waves)",
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
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(TableVariant {
                epoch: Instant::now(),
                mode,
                variant: name,
            })
        })
        .collect()
}

const VARIANTS: &[(AnimationMode, &str)] = &[
    (AnimationMode::Breathe, "Breathe (default)"),
    (AnimationMode::Sweep, "Sweep"),
    (AnimationMode::Plasma, "Plasma"),
];

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct TableVariant {
    epoch: Instant,
    mode: AnimationMode,
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
        let cols = [
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ];

        render_centered(
            SkeletonTable::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .columns(&cols)
                .rows(5),
            None,
            Some(Constraint::Length(5)),
            area,
            buf,
        );
    }
}
