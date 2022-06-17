use std::sync::{Arc};

use eframe::egui;
use egui::{mutex::RwLock, Ui, CentralPanel, ScrollArea, SidePanel, TopBottomPanel, Button};

use crate::{stats::AnalyzerStats, Scan, utils::bytes_to_human};

pub struct App {
    current_file: Arc<RwLock<Scan>>
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let r = self.current_file.read();
        let state = &*r;

        egui::CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("my_panel").show(ctx, |ui| {
                ui.heading("Disk Usage Analyzer");
            });
            SidePanel::left("my_left_panel").show(ctx, |ui| {
                let scan_button = Button::new("Scan");
                if ui.add_enabled(!state.completed_at.is_none(), scan_button).clicked() {
                }
                let stop_button = Button::new("Stop");
                if ui.add_enabled(state.completed_at.is_none(), stop_button).clicked() {
                }
            });
            CentralPanel::default().show(ctx, |ui| {

                if let Some(current_file) = &state.current_file {
                    ui.label(format!("Scanning {}", state.dir));
                    ui.label(format!("Usage (seen): {}", bytes_to_human(state.total_bytes)));
                    
                    ui.label(current_file.clone());

                    // Still scanning, so repaint
                    ctx.request_repaint();
                } else {
                    if let Some(completed_at) = state.completed_at {
                        ui.label(format!("Scanned {}", state.dir));
                        let duration = completed_at.signed_duration_since(state.started_at);
                        let duration_str = format!("{}:{}:{}", duration.num_hours(), duration.num_minutes(), duration.num_seconds());
                        ui.label(format!("Completed at {} (took {})", completed_at.format("%a %b %e %T %Y"), duration_str));
                    }
                    ui.label(format!("Total usage: {}", bytes_to_human(state.total_bytes)));

                    render_results(ui, state);
                }
            });

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

fn render_results(ui: &mut Ui, state: &Scan) {
    ui.separator();

    ui.label("Largest files:");

    ScrollArea::vertical().show(ui, |ui| {
        for file in state.largest_files.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("{} ({})", file.0, bytes_to_human(file.1)));

                if ui.button("Delete (trash)").clicked() {
                }
                if ui.button("Delete (force)").clicked() {
                }
            });
        }
    });
}

impl App {
    pub fn new(current_file: Arc<RwLock<Scan>>) -> Self {
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