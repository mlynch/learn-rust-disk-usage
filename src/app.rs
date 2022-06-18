use std::fs;

use std::{borrow::BorrowMut, cell::RefCell, rc::Rc, sync::Arc};

use eframe::egui;
use egui::{
    mutex::RwLock, vec2, Align, Align2, Button, CentralPanel, Layout, ScrollArea, SidePanel,
    TopBottomPanel, Ui, Visuals, Window,
};

use crate::{utils::bytes_to_human, Scan};

pub struct UiState {
    show_delete_confirm: bool,
    file_to_delete: Option<(String, bool)>,
}

pub struct App {
    current_file: Arc<RwLock<Scan>>,
    ui_state: RefCell<UiState>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());

        let r = self.current_file.read();
        let state = &*r;

        egui::CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("my_panel").show(ctx, |ui| {
                ui.heading("Disk Usage Analyzer");
            });
            SidePanel::left("my_left_panel").show(ctx, |ui| {
                let scan_button = Button::new("Scan");

                if ui
                    .add_enabled(!state.completed_at.is_none(), scan_button)
                    .clicked()
                {}
                let stop_button = Button::new("Stop");
                if ui
                    .add_enabled(state.completed_at.is_none(), stop_button)
                    .clicked()
                {}
            });
            CentralPanel::default().show(ctx, |ui| {
                if let Some(current_file) = &state.current_file {
                    ui.label(format!("Scanning {}", state.dir));
                    ui.label(format!(
                        "Usage (seen): {}",
                        bytes_to_human(state.total_bytes)
                    ));

                    ui.label(current_file.clone());

                    // Still scanning, so repaint
                    ctx.request_repaint();
                } else {
                    if let Some(completed_at) = state.completed_at {
                        ui.label(format!("Scanned {}", state.dir));
                        let duration = completed_at.signed_duration_since(state.started_at);
                        let duration_str = format!(
                            "{}:{}:{}",
                            duration.num_hours(),
                            duration.num_minutes(),
                            duration.num_seconds()
                        );
                        ui.label(format!(
                            "Completed at {} (took {})",
                            completed_at.format("%a %b %e %T %Y"),
                            duration_str
                        ));
                    }
                    ui.label(format!(
                        "Total usage: {}",
                        bytes_to_human(state.total_bytes)
                    ));

                    render_results(ui, ctx, state, &self.ui_state); //&mut self.show_delete_confirm);
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

fn delete_file(path: String, force: bool) {
    if force {
        match fs::remove_file(&path) {
            Ok(_) => {
                println!("Deleted!");
                // total_deleted += file.1;
            }
            Err(e) => println!("Unable to delete: {}", e),
        }
    } else {
        match trash::delete(&path) {
            Ok(_) => {
                println!("Deleted!");
                // total_deleted += file.1;
            }
            Err(e) => println!("Unable to delete: {}", e),
        }
    }
}

fn render_results(ui: &mut Ui, ctx: &egui::Context, state: &Scan, ui_state: &RefCell<UiState>) {
    //show_delete_confirm: &mut bool) {
    ui.separator();

    ui.label("Largest files:");

    let mut show_confirm = ui_state.borrow().show_delete_confirm.clone();

    confirm(
        ui,
        ctx,
        "Are you sure you want to delete that file?",
        &mut show_confirm,
        |confirm| {
            println!("Closing window here");
            let mut s = ui_state.borrow_mut();

            if confirm {
                if let Some(file_to_delete) = s.file_to_delete.clone() {
                    println!("Deleting file {} {}", file_to_delete.0, file_to_delete.1);
                    delete_file(file_to_delete.0, file_to_delete.1);
                }
            }

            s.file_to_delete = None;

            s.show_delete_confirm = false;
        },
    );

    ScrollArea::vertical().show(ui, |ui| {
        for file in state.largest_files.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("{} ({})", file.0, bytes_to_human(file.1)));

                if ui.button("Delete (trash)").clicked() {
                    // let mut s = ui_state.borrow_mut();
                    let mut s = ui_state.borrow_mut();
                    s.show_delete_confirm = true;
                    s.file_to_delete = Some((file.0.clone(), false));
                    // *show_delete_confirm = true;
                }
                if ui.button("Delete (force)").clicked() {
                    // let mut s = ui_state.borrow_mut();
                    let mut s = ui_state.borrow_mut();
                    s.show_delete_confirm = true;
                    s.file_to_delete = Some((file.0.clone(), true));
                    // *show_delete_confirm = true;
                }
            });
        }
    });
}

fn confirm<F>(ui: &mut Ui, ctx: &egui::Context, title: &str, open: &mut bool, close: F)
where
    F: FnOnce(bool),
{
    Window::new(title)
        // .open(&mut ui_state.borrow_mut().show_delete_confirm)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .open(open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(), |ui| {
                    // let mut s = ui_state.borrow_mut();
                    if ui.button("Confirm").clicked() {
                        close(true);
                    } else if ui.button("Cancel").clicked() {
                        // *open = false;
                        close(false);
                        // s.show_delete_confirm = false;
                    }
                });
            })
        });
}

impl App {
    pub fn new(current_file: Arc<RwLock<Scan>>) -> Self {
        let options = eframe::NativeOptions::default();
        let ui_state = RefCell::new(UiState {
            show_delete_confirm: false,
            file_to_delete: None,
        });

        let app = App {
            current_file,
            ui_state,
        };

        eframe::run_native("Disk Usage", options, Box::new(|_cc| Box::new(app)));
    }
}
