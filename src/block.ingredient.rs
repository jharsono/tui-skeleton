use std::time::Instant;

use ratatui_core::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

use super::SkeletonBlock;
use crate::AnimationMode;

const DESCRIPTION: &str =
    "Solid filled rectangle with animated brightness — the atomic skeleton unit.";

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
        name: "base",
        ty: "Color",
        description: "Dim resting color (default: DarkGray)",
    },
    PropInfo {
        name: "highlight",
        ty: "Color",
        description: "Peak brightness color (default: Gray)",
    },
];

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    let mut out: Vec<Box<dyn Ingredient>> = VARIANTS
        .iter()
        .map(|&(mode, braille, name)| -> Box<dyn Ingredient> {
            Box::new(BlockVariant {
                epoch: Instant::now(),
                mode,
                braille,
                variant: name,
            })
        })
        .collect();

    out.push(Box::new(BlockCompare {
        epoch: Instant::now(),
    }));

    out
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

// ── Mode variants ──

struct BlockVariant {
    epoch: Instant,
    mode: AnimationMode,
    braille: bool,
    variant: &'static str,
}

impl Ingredient for BlockVariant {
    fn group(&self) -> &str {
        "Block"
    }
    fn name(&self) -> &str {
        self.variant
    }
    fn source(&self) -> &str {
        "tui_skeleton::block"
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
            SkeletonBlock::new(elapsed_ms(self.epoch))
                .mode(self.mode)
                .braille(self.braille),
            None,
            Some(Constraint::Length(3)),
            area,
            buf,
        );
    }
}

// ── Compare (all seven stacked) ──

struct BlockCompare {
    epoch: Instant,
}

impl Ingredient for BlockCompare {
    fn group(&self) -> &str {
        "Block"
    }
    fn name(&self) -> &str {
        "Compare"
    }
    fn source(&self) -> &str {
        "tui_skeleton::block"
    }
    fn description(&self) -> &str {
        "All seven fill × animation combinations stacked."
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }
    fn animated(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        use ratatui_core::text::Line;

        let ms = elapsed_ms(self.epoch);

        // 7 rows: label + bar + gap each, minus trailing gap + top/bottom fill.
        let constraints: Vec<Constraint> = std::iter::once(Constraint::Fill(1))
            .chain(VARIANTS.iter().enumerate().flat_map(|(i, _)| {
                let row = [Constraint::Length(1), Constraint::Length(1)];

                if i < VARIANTS.len() - 1 {
                    vec![row[0], row[1], Constraint::Length(1)]
                } else {
                    vec![row[0], row[1]]
                }
            }))
            .chain(std::iter::once(Constraint::Fill(1)))
            .collect();

        let areas = Layout::vertical(constraints).split(area);

        // Skip the leading Fill(1), then stride through label/bar/gap triples.
        for (i, &(mode, braille, name)) in VARIANTS.iter().enumerate() {
            let base = 1 + i * 3;
            let label_area = areas[base];
            let bar_area = areas[base + 1];

            Line::raw(name).render(label_area, buf);

            SkeletonBlock::new(ms)
                .mode(mode)
                .braille(braille)
                .render(bar_area, buf);
        }
    }
}
