#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use gaitool_rs::core::split::split;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use eframe::egui;
fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 400., y: 140. }),
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(
        "Split Gait",
        options,
        Box::new(|cc| Box::new(AppState::new(cc))),
    );
}

#[derive(PartialEq, Clone)]
struct ProcState {
    is_running: bool,
    percentage: f32,
    msg: Option<String>,
}

struct Proc {
    state: Arc<Mutex<ProcState>>,
    sx: std::sync::mpsc::Sender<ProcState>,
}

struct AppState {
    slider_value: u32,
    process: Proc,
    picked_dir: Option<String>,
    sub_dirs: Vec<PathBuf>,
}

impl AppState {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (sx, rx) = std::sync::mpsc::channel();
        let state = Arc::new(Mutex::new(ProcState {
            is_running: false,
            percentage: 0.,
            msg: None,
        }));
        spawn_repaint_thread(rx, state.clone(), cc.egui_ctx.clone());
        let process = Proc { state, sx };

        Self {
            slider_value: 70,
            process,
            picked_dir: None,
            sub_dirs: Vec::new(),
        }
    }
}

impl eframe::App for AppState {
    fn update(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        load_fonts(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Folder: ");
                    ui.vertical_centered_justified(|ui| {
                        let btn_label =
                            if let Some(picked_dir) = &self.picked_dir {
                                picked_dir
                            } else {
                                "Open or Drag Folder Here…"
                            };
                        if ui.button(btn_label).clicked() {
                            if let Some(path) =
                                rfd::FileDialog::new().pick_folder()
                            {
                                self.picked_dir =
                                    Some(path.display().to_string());
                                self.sub_dirs = std::fs::read_dir(path)
                                    .unwrap()
                                    .into_iter()
                                    .filter(|d| {
                                        d.as_ref()
                                            .unwrap()
                                            .metadata()
                                            .unwrap()
                                            .is_dir()
                                    })
                                    .map(|d| d.unwrap().path())
                                    .collect();
                            }
                        }
                    });
                });
                ui.vertical_centered_justified(|ui| {
                    ui.style_mut().spacing.slider_width = 300.;
                    ui.add(
                        egui::Slider::new(&mut self.slider_value, 0..=100)
                            .text("%"),
                    );
                })
            });
            let p_state = self.process.state.lock().unwrap();
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Status: ");
                    ui.add_visible_ui(p_state.is_running, |ui| {
                        if let Some(dir) = &p_state.msg {
                            ui.label(dir);
                        }
                    });
                });
                ui.vertical_centered(|ui| {
                    ui.add_visible_ui(p_state.is_running, |ui| {
                        ui.add(egui::ProgressBar::new(p_state.percentage));
                    });
                });
            });
            ui.vertical_centered(|ui| {
                ui.add_enabled_ui(
                    !p_state.is_running && self.picked_dir.is_some(),
                    |ui| {
                        if ui.button(" Start ").clicked() {
                            long_process(
                                self.process.sx.clone(),
                                self.sub_dirs.clone(),
                                self.slider_value as usize,
                            );
                        }
                    },
                );
            })
        });
        if !ctx.input().raw.dropped_files.is_empty() {
            self.picked_dir = Some(
                ctx.input().raw.dropped_files[0]
                    .clone()
                    .path
                    .unwrap()
                    .display()
                    .to_string(),
            );
        }
    }
}

fn long_process(
    sender: std::sync::mpsc::Sender<ProcState>,
    dirs: Vec<PathBuf>,
    percent: usize,
) {
    std::thread::spawn(move || {
        let mut p_state = ProcState {
            is_running: true,
            percentage: 0.,
            msg: None,
        };
        let num_works = dirs.len() + 1;
        for (i, dir) in dirs.iter().enumerate() {
            p_state.percentage = (i + 1) as f32 / num_works as f32;
            p_state.msg = Some(dir.display().to_string());
            sender.send(p_state.clone()).unwrap();

            let output_dir = dir.parent().unwrap().join("output");
            for file in std::fs::read_dir(dir).unwrap() {
                let file = file.unwrap().path();
                if let Err(e) = split(
                    &file,
                    &output_dir,
                    percent,
                    &PathBuf::from("./assets"),
                    None,
                ) {
                    println!("{}", e);
                    p_state.msg = Some(e.to_string());
                    sender.send(p_state.clone()).unwrap();
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        p_state.percentage = 1.;
        sender.send(p_state.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));

        p_state.msg = Some("Finished".to_string());
        sender.send(p_state.clone()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        p_state.is_running = false;
        sender.send(p_state.clone()).unwrap();
    });
}

fn spawn_repaint_thread<T: std::marker::Send + 'static>(
    rx: std::sync::mpsc::Receiver<T>,
    proc_state: Arc<Mutex<T>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || loop {
        if let Ok(state) = rx.recv() {
            *proc_state.lock().unwrap() = state;
            ctx.request_repaint();
        }
    });
}

fn load_fonts(ctx: &eframe::egui::Context) {
    let mut font = egui::FontDefinitions::default();
    font.font_data.insert(
        String::from("chinese_fallback"),
        egui::FontData::from_static(include_bytes!(
            "../assets/NotoSansTC-Regular.otf"
        )),
    );
    font.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .push("chinese_fallback".to_owned());
    ctx.set_fonts(font);
}
