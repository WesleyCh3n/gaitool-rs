use eframe::egui::{
    self,
    plot::{BoxElem, BoxPlot, BoxSpread, Legend, Line, Plot, PlotPoints},
    ScrollArea,
};

use crate::{
    config::{Position, Variable},
    data_process::{extract_info, RawData},
};

pub struct State {
    pub side_panel_open: bool,
    unprocess_dialog: bool,
}

pub struct FileList {
    path: String,
    raw_data: RawData,
    is_selected: bool,
}

pub struct Chart {
    pos: Position,
    var: Variable,
    // pos_selected: Option<usize>,
    // var_selected: Option<usize>,
    file_list: Vec<FileList>,
    show_boxplot: bool,
    show_lineplot: bool,
    pub state: State,
}

impl Chart {
    pub fn new() -> Self {
        Self {
            pos: Position::L,
            var: Variable::AccelX,
            // pos_selected: None,
            // var_selected: None,
            file_list: Vec::new(),
            show_boxplot: false,
            show_lineplot: false,
            state: State {
                side_panel_open: false,
                unprocess_dialog: false,
            },
        }
    }

    pub fn open_dir(&mut self) {
        let Self {
            file_list,
            state:
                State {
                    side_panel_open,
                    unprocess_dialog,
                },
            ..
        } = self;
        file_list.clear();
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            for entries in std::fs::read_dir(path).unwrap() {
                if let Ok(entry) = entries {
                    let info = extract_info(entry.path());
                    if info[0].len() != 12 || info[1].len() != 12 {
                        *unprocess_dialog = true;
                        *side_panel_open = false;
                        return;
                    }
                    if info[0][5] != String::from("exported with version") {
                        *unprocess_dialog = true;
                        *side_panel_open = false;
                        return;
                    }
                    println!("{:?}", info);
                    file_list.push(FileList {
                        path: entry.file_name().to_str().unwrap().to_string(),
                        raw_data: RawData::new(entry.path()),
                        is_selected: false,
                    })
                }
            }
            file_list[0].is_selected = true;
            // get data
            *side_panel_open = true;
        }
    }
}

impl super::View for Chart {
    fn show(&mut self, ctx: &eframe::egui::Context) {
        if self.state.side_panel_open {
            egui::SidePanel::right("right files panel")
                // .resizable(true)
                .show(ctx, |ui| {
                    side_panel_ui(self, ui);
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            chart_ui(self, ui);
        });

        if self.state.unprocess_dialog {
            egui::Window::new("unprocess_dialog")
                .collapsible(false)
                .title_bar(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                .show(ctx, |ui| {
                    ui.label("hello");
                    ui.horizontal(|ui| {
                        if ui.button("Ok").clicked() {};
                        if ui.button("Cancel").clicked() {
                            self.state.unprocess_dialog = false;
                        };
                    })
                });
        }
    }
}

fn chart_ui(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        show_boxplot,
        show_lineplot,
        file_list,
        pos,
        var,
        ..
    } = app;
    if !*show_boxplot && !*show_lineplot {
        ui.centered_and_justified(|ui| {
            ui.label("Please select one chart");
        });
    }
    if file_list.is_empty() {
        return;
    }

    let y = ui.available_height();

    if *show_boxplot {
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
            .height(if !*show_lineplot { y } else { y / 3. })
            .include_x(0.)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.box_plot(box1);
                plot_ui.box_plot(box2);
            });
    }
    if *show_lineplot {
        Plot::new("my_plot")
            .legend(Legend::default()) // with .name() method
            .height(if !*show_boxplot { y } else { y * 2. / 3. })
            .include_x(0.) // show x axis label
            .show(ui, |plot_ui| {
                for f in file_list {
                    if f.is_selected {
                        let (x, y) = (
                            &f.raw_data.x,
                            &f.raw_data.y.get(pos).unwrap().get(var).unwrap(),
                        );
                        let data: PlotPoints = x
                            .into_iter()
                            .zip(y.into_iter())
                            .map(|(a, b)| [a.clone(), b.clone()])
                            .collect();
                        plot_ui.line(Line::new(data).name(&*f.path));
                    }
                }
            });
    }
}

fn side_panel_ui(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        show_boxplot,
        show_lineplot,
        // pos_list,
        // pos_selected,
        // var_list,
        // var_selected,
        file_list,
        pos,
        var,
        ..
    } = app;
    egui::Grid::new("options")
        .max_col_width(100.)
        .striped(true)
        .show(ui, |ui| {
            ui.heading("Options");
            ui.end_row();
            ui.label("Position: ");
            egui::ComboBox::from_id_source("combo")
                .selected_text(format!("{:?}", pos))
                .show_ui(ui, |ui| {
                    Position::iterator().for_each(|p| {
                        ui.selectable_value(pos, p.clone(), format!("{:?}", p));
                    });
                });
            ui.end_row();
            ui.label("Variable: ");
            egui::ComboBox::from_id_source("var_combo")
                .selected_text(format!("{:?}", var))
                .show_ui(ui, |ui| {
                    Variable::iterator().for_each(|v| {
                        ui.selectable_value(var, v.clone(), format!("{:?}", v));
                    });
                });
            ui.end_row();
            ui.heading("Plots");
            ui.end_row();
            ui.label("show box plots: ");
            ui.vertical(|ui| {
                ui.checkbox(show_boxplot, "show box plot");
            });
            ui.end_row();
            ui.label("show line plot: ");
            ui.vertical(|ui| {
                ui.checkbox(show_lineplot, "show line plot");
            });
        });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    ui.heading("File List");
    ui.group(|ui| {
        ScrollArea::horizontal()
            .vscroll(true)
            // .max_height(200.0)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Grid::new("files").striped(true).num_columns(1).show(
                    ui,
                    |ui| {
                        for row in file_list {
                            ui.checkbox(
                                &mut row.is_selected,
                                row.path.as_str(),
                            );
                            ui.end_row();
                        }
                    },
                );
            });
    });
}
