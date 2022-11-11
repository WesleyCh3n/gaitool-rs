use eframe::egui::{
    self,
    plot::{BoxElem, BoxPlot, BoxSpread, Legend, Line, Plot, PlotPoints},
    ScrollArea,
};

#[derive(PartialEq, Debug)]
pub enum Enum {
    First,
    Second,
    Third,
}

pub struct Chart {
    radio: Enum,
    selected_file_index: usize,
    show_boxplot: bool,
    show_lineplot: bool,
}

impl Chart {
    pub fn new() -> Self {
        Self {
            radio: Enum::First,
            selected_file_index: 0,
            show_boxplot: true,
            show_lineplot: true,
        }
    }
}

impl super::View for Chart {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.label("here i want box chart and line chart");
        ui.vertical_centered_justified(|ui| {
            let box1 = BoxPlot::new(vec![
                BoxElem::new(0.5, BoxSpread::new(1.5, 2.2, 2.5, 2.6, 3.1))
                    .name("Day 1"),
                BoxElem::new(2.5, BoxSpread::new(0.4, 1.0, 1.1, 1.4, 2.1))
                    .name("Day 2"),
                BoxElem::new(4.5, BoxSpread::new(1.7, 2.0, 2.2, 2.5, 2.9))
                    .name("Day 3"),
            ])
            .name("Experiment A");
            let box2 = BoxPlot::new(vec![BoxElem::new(
                1.0,
                BoxSpread::new(0.2, 0.5, 1.0, 2.0, 2.7),
            )
            .name("Day 2")])
            .name("Experiment B");
            Plot::new("Box Plot Demo")
                .view_aspect(4.0)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    plot_ui.box_plot(box1);
                    plot_ui.box_plot(box2);
                });
        });
        let sin: PlotPoints = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x, x.sin()]
            })
            .collect();
        let cos: PlotPoints = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.01;
                [x, x.cos()]
            })
            .collect();
        Plot::new("my_plot")
            .legend(Legend::default()) // with .name() method
            .include_x(0.) // show x axis label
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(sin).name("sin"));
                plot_ui.line(Line::new(cos).name("cos"));
            });
    }
    fn show(&mut self, ctx: &eframe::egui::Context) {
        let Self {
            radio,
            selected_file_index,
            show_boxplot,
            show_lineplot,
        } = self;
        egui::SidePanel::right("right files panel").show(ctx, |ui| {
            egui::Grid::new("options")
                .max_col_width(100.)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("some option tools");
                    ui.end_row();
                    ui.label(
                        "i think combox box should dynamic read from vec name",
                    );
                    ui.end_row();
                    ui.label("combobox: ");
                    egui::ComboBox::from_id_source("combo")
                        .selected_text(format!("{:?}", radio))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(radio, Enum::First, "First");
                            ui.selectable_value(radio, Enum::Second, "Second");
                            ui.selectable_value(radio, Enum::Third, "Third");
                        });
                    ui.end_row();
                    ui.separator();
                    ui.separator();
                    ui.end_row();
                    ui.label("show box plots: ");
                    ui.vertical(|ui| {
                        ui.checkbox(show_boxplot, "show box plot");
                    });
                    ui.end_row();
                    ui.label("show line plot: ");
                    ui.vertical(|ui| {
                        ui.checkbox(show_lineplot, "show box plot");
                    });
                });
            ui.separator();
            ui.label("a scroll view of file list");
            ui.group(|ui| {
                ScrollArea::vertical()
                    // .max_height(200.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for row in 0..26 {
                            let text =
                                format!("This is row {}/{}", row + 1, 100);
                            ui.selectable_value(selected_file_index, row, text);
                        }
                    });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}
