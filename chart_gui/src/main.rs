// #![windows_subsystem = "windows"]

mod app;
mod chart;
mod config;
mod data_process;

use app::App;

pub trait View {
    fn show(&mut self, ctx: &eframe::egui::Context);
}

fn main() {
    let options = eframe::NativeOptions {
        // initial_window_size: Some(egui::Vec2 { x: 400., y: 160. }),
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(
        "Gaitool",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
