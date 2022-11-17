use eframe::egui;

use crate::chart::Chart;

pub struct App {
    chart: Chart,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        App {
            chart: Chart::new(cc),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open File").clicked() {}
                    if ui.button("Open Folder").clicked() {
                        self.chart.open_dir();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Close Folder").clicked() {
                        self.chart.close_dir();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        frame.close();
                    }
                });
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.toggle_value(
                            &mut self.chart.state.show_side_panel,
                            "Tools",
                        );
                        ui.separator();
                    },
                );
            })
        });
        use super::View as _;
        self.chart.show(ctx);
    }
}
