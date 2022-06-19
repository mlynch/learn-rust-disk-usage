use egui::{Widget, Vec2, Sense, Color32};

pub struct PieChart {

}

impl PieChart {
    pub fn new() -> Self {
        PieChart {
        }
    }
}

impl Widget for PieChart {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = Vec2::new(256.0, 256.0);

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::hover());

        if ui.is_rect_visible(rect) {
            let center = rect.center();
            let radius = 0.5 * rect.height();

            ui.painter().circle_filled(center, radius, Color32::from_rgb(125, 125, 125));
        }

        response
    }
}