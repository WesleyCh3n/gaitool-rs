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
                    file_list.push(FileList {
                        path: entry.file_name().to_str().unwrap().to_string(),
                        raw_data: RawData::new(entry.path()),
                        is_selected: false,
                    })
                }
            }
            file_list[0].is_selected = true;
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
        Plot::new("Box Plot")
            .height(if !*show_lineplot { y } else { y / 3. })
            .include_x(0.)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                let mut i = 0.;
                for f in file_list.into_iter() {
                    if f.is_selected {
                        let (_, min, q1, mid, q3, max) =
                            &f.raw_data.y.get(pos).unwrap().get(var).unwrap();
                        plot_ui.box_plot(
                            BoxPlot::new(vec![BoxElem::new(
                                i,
                                BoxSpread::new(
                                    min.clone(),
                                    q1.clone(),
                                    mid.clone(),
                                    q3.clone(),
                                    max.clone(),
                                ),
                            )])
                            .name(&*f.path),
                        );
                        i += 0.5;
                    }
                }
            });
    }
    if *show_lineplot {
        Plot::new("Line Plot")
            .legend(Legend::default()) // with .name() method
            .height(if !*show_boxplot { y } else { y * 2. / 3. })
            .include_x(0.) // show x axis label
            .show(ui, |plot_ui| {
                for f in file_list {
                    if f.is_selected {
                        let (x, (y, min, _, _, _, max)) = (
                            &f.raw_data.x,
                            &f.raw_data.y.get(pos).unwrap().get(var).unwrap(),
                        );
                        plot_ui.line(
                            Line::new(
                                x.into_iter()
                                    .zip((&f.raw_data.l_contact).into_iter())
                                    .map(|(a, b)| {
                                        let upper_bound = if b.gt(&0) {
                                            max.clone()
                                        } else {
                                            min.clone()
                                        };
                                        [a.clone(), upper_bound]
                                    })
                                    .collect::<PlotPoints>(),
                            )
                            .fill(*min as f32)
                            .color(egui::Color32::LIGHT_BLUE)
                            .width(0.)
                            .name(format!("{} L Contact", f.path)),
                        );
                        plot_ui.line(
                            Line::new(
                                x.into_iter()
                                    .zip((&f.raw_data.r_contact).into_iter())
                                    .map(|(a, b)| {
                                        let upper_bound = if b.gt(&0) {
                                            max.clone()
                                        } else {
                                            min.clone()
                                        };
                                        [a.clone(), upper_bound]
                                    })
                                    .collect::<PlotPoints>(),
                            )
                            .fill(*min as f32)
                            .color(egui::Color32::LIGHT_GREEN)
                            .width(0.)
                            .name(format!("{} R Contact", f.path)),
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
