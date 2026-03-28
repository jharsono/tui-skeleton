use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{layout::render_centered, Ingredient, PropInfo};

use super::SkeletonList;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Short spaced items with ragged right edges, mimicking a menu or sidebar.";

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
        name: "items",
        ty: "u16",
        description: "Number of list items (default: 5)",
    },
    PropInfo {
        name: "widths",
        ty: "&[f32]",
        description: "Per-item width fractions, cycling",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, name)| -> Box<dyn Ingredient> {
            Box::new(ListVariant {
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

struct ListVariant {
    epoch: Instant,
    mode: AnimationMode,
    variant: &'static str,
}

impl Ingredient for ListVariant {
    fn group(&self) -> &str {
        "List"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::list"
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
            SkeletonList::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .items(5),
            None,
            Some(Constraint::Length(9)),
            area,
            buf,
        );
    }
}
