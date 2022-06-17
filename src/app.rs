use std::sync::{mpsc::Receiver, Arc};

use eframe::egui;
use egui::mutex::RwLock;

use crate::stats::AnalyzerStats;

pub struct App {
    current_file: Arc<RwLock<String>>
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disk Usage Analyzer");

            let r = self.current_file.read();

            ui.label((*r).clone());

            ctx.request_repaint();
            /*
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
            */
        });
    }
}

impl App {
    pub fn new(current_file: Arc<RwLock<String>>) -> Self {
        let options = eframe::NativeOptions::default();
        let app = App {
            current_file
        };

        eframe::run_native(
            "Disk Usage",
            options,
            Box::new(|_cc| Box::new(app)),
        );
    }

    pub fn set_dir(&self, dir: String) {
    }

    pub fn render_stats(&self, stats: &AnalyzerStats) {
    }
}