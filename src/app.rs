use std::fs;

use std::sync::atomic::AtomicBool;
use std::{cell::RefCell, sync::Arc, thread};

use chrono::{Local, DateTime};
use dirs::home_dir;
use egui::{Vec2, Frame, Context};
use egui_extras::{Size, TableBuilder};

use eframe::egui;
use egui::{
    mutex::RwLock, vec2, Align2, Button, CentralPanel, Layout, ScrollArea, SidePanel,
    TopBottomPanel, Ui, Visuals, Window,
};
use rfd::{AsyncFileDialog, FileDialog};

use crate::analyzer::{Analyzer, ScanSettings};
use crate::{utils::bytes_to_human};

#[derive(PartialEq, Clone)]
#[repr(u8)]
enum CurrentTab {
    LargeFiles = 0,
    Recs = 1,
    Summary = 2
}

#[derive(Clone)]
pub struct UiState {
    show_delete_confirm: bool,
    show_settings: RefCell<bool>,
    file_to_delete: Option<(String, bool)>,
    current_tab: CurrentTab,
    setting_root_dir: RefCell<String>,
    setting_ignore_glob: RefCell<String>,
    setting_developer_cache_dirs: RefCell<String>,
    setting_hidden: RefCell<bool>,
    setting_nlargest: RefCell<u64>,
    setting_largebytes: RefCell<u64>
}

type LargeFile = (String, u64);
pub struct Scan {
    pub dir: String,
    pub started_at: DateTime<Local>,
    pub completed_at: Option<DateTime<Local>>,
    pub current_file: Option<String>,
    pub total_bytes: u64,
    pub largest_files: Box<Vec<LargeFile>>,
    pub total_music: u64,
    pub total_images: u64,
    pub total_videos: u64,
    pub total_documents: u64,
    pub total_binaries: u64,
    pub total_archives: u64,
    pub total_other: u64,
    pub dev_total_usage: u64,
    pub developer_dirs: Vec<LargeFile>
}

impl Scan {
    pub fn clear(&mut self) {
        self.dir = String::from("");
        self.started_at = Local::now();
        self.completed_at = None;
        self.current_file = None;
        self.total_bytes = 0;
        self.largest_files = Box::new(vec![]);
        self.total_music = 0;
        self.total_images = 0;
        self.total_videos = 0;
        self.total_documents = 0;
        self.total_archives= 0;
        self.total_other = 0;
        self.dev_total_usage = 0;
        self.developer_dirs = vec![];
    }
}

pub struct App {
    scan_results: Arc<RwLock<Scan>>,
    ui_state: RefCell<UiState>,
    scanning: Arc<RwLock<bool>>
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(Visuals::dark());

        let r = self.scan_results.read();
        let scan_results = &*r;

        egui::CentralPanel::default().show(ctx, |ui| {
            if *self.ui_state.borrow().show_settings.borrow() {
                render_settings(ui, ctx, scan_results, &self.ui_state);
            }

            TopBottomPanel::top("my_panel")
            .frame(Frame::group(ui.style()).inner_margin(Vec2::new(8.0, 16.0)))
            .show(ctx, |ui| {
                ui.heading("Disk Usage Analyzer");
            });
            CentralPanel::default().show(ctx, |ui| {
                render_scan_control(ui, ctx, &self, &self.ui_state);

                if let Some(current_file) = &scan_results.current_file {
                    let duration = Local::now().signed_duration_since(scan_results.started_at);
                    let duration_str = format!(
                        "{}:{}:{}",
                        duration.num_hours(),
                        duration.num_minutes(),
                        duration.num_seconds()
                    );
                    ui.label(format!("Scanning {}", scan_results.dir));
                    ui.label(format!(
                        "Elapsed time: {}",
                        duration_str
                    ));
                    ui.label(format!(
                        "Usage (seen): {}",
                        bytes_to_human(scan_results.total_bytes)
                    ));

                    ui.label(current_file.clone());

                    // Still scanning, so repaint
                    if *self.scanning.read() {
                        ctx.request_repaint();
                    }
                } else {
                    if let Some(completed_at) = scan_results.completed_at {
                        ui.label(format!("Scanned {}", scan_results.dir));
                        let duration = completed_at.signed_duration_since(scan_results.started_at);
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
                        ui.label(format!(
                            "Total usage: {}",
                            bytes_to_human(scan_results.total_bytes)
                        ));
                    }

                    render_results(ui, ctx, scan_results, &self.ui_state); //&mut self.show_delete_confirm);
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

fn render_scan_control(ui: &mut Ui, ctx: &egui::Context, app: &App, ui_state: &RefCell<UiState>) {
    let scan_button = Button::new("Scan");

    {
        let state = ui_state.borrow();
        let mut dir = state.setting_root_dir.borrow_mut();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut *dir);
            if ui.button("...").clicked() {
                let f = FileDialog::new()
                    .set_directory(&*dir)
                    .pick_folder();

                if let Some(folder) = f {
                    println!("Got folder: {:?}", folder.to_str());

                    *dir = String::from(folder.to_str().unwrap());
                }
                // let data = file.unwrap().read().await;

            }
        });
    }


    if ui
        .add_enabled(!*app.scanning.write(), scan_button)
        .clicked()
    {
        app.start_scan();
    }
    let stop_button = Button::new("Stop");
    if ui
        .add_enabled(*app.scanning.write(), stop_button)
        .clicked()
    {
        app.stop_scan();
    }
            /*
            SidePanel::left("my_left_panel")
            .frame(Frame::group(ui.style()).inner_margin(Vec2::new(8.0, 8.0)))
            .show(ctx, |ui| {
                ui.columns(2, |cols| {
                });

                if ui.button("Settings").clicked() {
                    let state = self.ui_state.borrow();
                    let mut s = state.show_settings.borrow_mut();
                    *s = true;
                }
            });
            */
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

    let mut s = ui_state.borrow_mut();

    ui.horizontal(|ui| {
        if ui.selectable_value(&mut s.current_tab, CurrentTab::LargeFiles, "Large files").clicked() {
            //let mut s = ui_state.borrow_mut();
            s.current_tab = CurrentTab::LargeFiles;
        }
        if ui.selectable_value(&mut s.current_tab, CurrentTab::Recs, "Recommendations").clicked() {
            //let mut s = ui_state.borrow_mut();
            s.current_tab = CurrentTab::Recs;
        };
        if ui.selectable_value(&mut s.current_tab, CurrentTab::Summary, "Summary").clicked() {
            //let mut s = ui_state.borrow_mut();
            s.current_tab = CurrentTab::Summary;
        }
    });

    let mut show_confirm = s.show_delete_confirm.clone();

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

    // Drop our mutable reference to ui_state
    drop(s);

    let uis = ui_state.borrow();
    match uis.current_tab {
        CurrentTab::LargeFiles => render_large_files(ui, ctx, state, ui_state),
        CurrentTab::Recs => render_recs(ui, ctx, state, ui_state),
        CurrentTab::Summary => render_summary(ui, ctx, state, ui_state),
    }
}

fn render_large_files(ui: &mut Ui, _ctx: &egui::Context, state: &Scan, ui_state: &RefCell<UiState>) {
    ScrollArea::vertical().show(ui, |ui| {
        if state.largest_files.len() == 0 {
            let s = ui_state.borrow();
            ui.label(format!("No large files detected (> {})", bytes_to_human(*s.setting_nlargest.borrow())));
        }
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right().with_cross_align(egui::Align::Center))
            .column(Size::remainder().at_least(400.0))
            .column(Size::initial(110.0).at_least(90.0))
            .column(Size::initial(110.0).at_least(90.0))
            .resizable(true)
            .body(|mut body| {
                for file in state.largest_files.iter() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{} ({})", file.0, bytes_to_human(file.1)));
                        });

                        row.col(|ui| {
                            if ui.button("Delete (trash)").clicked() {
                                // let mut s = ui_state.borrow_mut();
                                let mut s = ui_state.borrow_mut();
                                s.show_delete_confirm = true;
                                s.file_to_delete = Some((file.0.clone(), false));
                                // *show_delete_confirm = true;
                            }
                        });

                        row.col(|ui| {
                            if ui.button("Delete (force)").clicked() {
                                // let mut s = ui_state.borrow_mut();
                                let mut s = ui_state.borrow_mut();
                                s.show_delete_confirm = true;
                                s.file_to_delete = Some((file.0.clone(), true));
                                // *show_delete_confirm = true;
                            }
                        });
                    });
                }
            });
    });
}

fn render_recs(ui: &mut Ui, ctx: &egui::Context, scan_results: &Scan, ui_state: &RefCell<UiState>) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Developer Directories");
        ui.label("These directories contain locally-synced installation files created while developing software. In most cases they can be safely deleted as they will be re-created when needed.");
        ui.label(format!("Detected {}", bytes_to_human(scan_results.dev_total_usage)));
        if ui.button("Delete all").clicked() {
        }
    });
}

fn render_summary(ui: &mut Ui, ctx: &egui::Context, scan_results: &Scan, ui_state: &RefCell<UiState>) {
}

fn render_settings(ui: &mut Ui, ctx: &egui::Context, scan_results: &Scan, ui_state: &RefCell<UiState>) {
    let state = ui_state.borrow();
    let mut open = state.show_settings.borrow_mut();

    Window::new("Settings")
        // .open(&mut ui_state.borrow_mut().show_delete_confirm)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    let mut setting_developer_cache_dirs = state.setting_developer_cache_dirs.borrow_mut();
                    let mut setting_ignore_glob = state.setting_ignore_glob.borrow_mut();

                    ui.label("Developer cache dirs glob");
                    ui.text_edit_singleline(&mut *setting_developer_cache_dirs);
                    ui.end_row();

                    ui.label("Ignore dirs");
                    ui.text_edit_singleline(&mut *setting_ignore_glob);
                    ui.end_row();
                });
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
    pub fn new() -> Self {
        // let home = home_dir();

        let mut options = eframe::NativeOptions::default();
        options.initial_window_size = Some(Vec2::new(1024.0, 768.0));
        let ui_state = RefCell::new(UiState {
            show_delete_confirm: false,
            show_settings: RefCell::new(false),
            file_to_delete: None,
            current_tab: CurrentTab::LargeFiles,
            setting_developer_cache_dirs: RefCell::new(String::from("**/node_modules")),
            setting_ignore_glob: RefCell::new(String::from("")),
            setting_hidden: RefCell::new(true),
            setting_largebytes: RefCell::new(1024 * 1024 * 50),
            setting_nlargest: RefCell::new(100),
            setting_root_dir: RefCell::new(String::from("/Users/max/hack/usage-test"))
        });

        let scan_results = Arc::new(RwLock::new(Scan {
            dir: String::from(""),
            started_at: Local::now(),
            completed_at: None,
            current_file: None,
            total_bytes: 0,
            largest_files: Box::new(vec![]),
            total_music: 0,
            total_images: 0,
            total_videos: 0,
            total_documents: 0,
            total_binaries: 0,
            total_archives: 0,
            total_other: 0,
            dev_total_usage: 0,
            developer_dirs: vec![],
        }));

        let app = App {
            scan_results,
            ui_state,
            scanning: Arc::new(RwLock::new(false))
        };

        eframe::run_native("Disk Usage", options, Box::new(|_cc| Box::new(app)));
    }

    fn start_scan(&self) {
        // self.scanning = RefCell::new(true);
        let mut is_scanning = self.scanning.write();
        *is_scanning = true;
        // let mut is_scanning = self.scanning.borrow_mut();
        //*is_scanning = true;

        let producer_lock = self.scan_results.clone();

        let state = self.ui_state.borrow().clone();

        let scanning_arc = self.scanning.clone();

        let _handle = thread::spawn(move || {
            // let _ = set_current_thread_priority(ThreadPriority::Min) as Result<(), _>;
            // let cloned_context = ctx.clone();

            let mut w = producer_lock.write();
            w.clear();
            w.dir = (*state.setting_root_dir.borrow()).clone();
            drop(w);

            let settings = ScanSettings {
                ignore: (*state.setting_ignore_glob.borrow()).clone(),
                dir: (*state.setting_root_dir.borrow()).clone(),
                nlargest: *state.setting_nlargest.borrow(),
                largebytes: *state.setting_largebytes.borrow(),
                hidden: *state.setting_hidden.borrow(),
            };

            let analyzer = Analyzer::new(&settings, producer_lock);

            analyzer.analyze().expect("Unable to read file or directory");

            let mut is_scanning = scanning_arc.write();
            *is_scanning = false;
        });
    }

    fn stop_scan(&self) {
        let mut is_scanning = self.scanning.write();
        *is_scanning = false;
    }
}
