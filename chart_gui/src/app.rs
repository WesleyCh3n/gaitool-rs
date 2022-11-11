use eframe::egui;

use crate::chart::Chart;

pub struct App {
    chart: Chart,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        App {
            chart: Chart::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open File").clicked() {}
                    if ui.button("Open Folder").clicked() {}
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        frame.close();
                    }
                });
            })
        });
        use super::View as _;
        self.chart.show(ctx);
    }
}
