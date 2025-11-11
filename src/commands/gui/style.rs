
use egui::{
    style::{Margin, Strafes, Style, Visuals},
    Color32, Stroke,
};

pub fn apicentric_style() -> Style {
    Style {
        visuals: Visuals {
            dark_mode: true,
            panel_fill: Color32::from_rgb(33, 37, 41),
            window_fill: Color32::from_rgb(33, 37, 41),
            window_stroke: Stroke::new(1.0, Color32::from_rgb(73, 80, 87)),
            ..Visuals::dark()
        },
        ..Default::default()
    }
}
