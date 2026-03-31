use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Rect},
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

use super::SkeletonKvTable;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Key-value pairs with fixed key column and variable value widths, mimicking a detail panel.";

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
        name: "pairs",
        ty: "u16",
        description: "Number of key-value pairs (default: 5)",
    },
    PropInfo {
        name: "key_width",
        ty: "u16",
        description: "Fixed width of the key column (default: 12)",
    },
    PropInfo {
        name: "value_widths",
        ty: "&[f32]",
        description: "Per-pair value width fractions, cycling",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    VARIANTS
        .iter()
        .map(|&(mode, braille, name)| -> Box<dyn Ingredient> {
            Box::new(KvTableVariant {
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

fn elapsed_ms(epoch: Instant) -> u64 {
    epoch.elapsed().as_millis() as u64
}

struct KvTableVariant {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl Ingredient for KvTableVariant {
    fn group(&self) -> &str {
        "Key-Value Table"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::kv_table"
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
            SkeletonKvTable::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .braille(self.braille)
                .pairs(5),
            None,
            Some(Constraint::Length(9)),
            area,
            buf,
        );
    }
}
