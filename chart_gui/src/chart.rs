use std::sync::{Arc, Mutex};

use eframe::egui::{
    self,
    plot::{
        BoxElem, BoxPlot, BoxSpread, Legend, Line, Plot, PlotPoints, VLine,
    },
    ScrollArea,
};

use crate::{
    config::{Position, Variable},
    data_process::{DataInfo, Manager, Message},
};

#[derive(PartialEq)]
enum BoxData {
    MinMax,
    Support,
}

pub struct State {
    pub show_side_panel: bool,
    pub unprocess_dialog: bool,
    show_process_win: bool,
    show_boxplot: bool,
    box_data: BoxData,
    show_min: bool,
    show_max: bool,
    show_gait: bool,
    show_db: bool,
    show_lt: bool,
    show_rt: bool,
    show_lineplot: bool,
    show_gait_line: bool,
    show_contact: bool,
}

pub struct Chart {
    pos: Position,
    var: Variable,
    manager: Manager,
    result: Arc<Mutex<Message>>,
    file_selects: Vec<bool>,
    pub state: State,
}

impl Chart {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (sx, rx) = std::sync::mpsc::channel();
        let manager = Manager::new(sx);
        let result = Arc::new(Mutex::new(Message::Nothing));
        spawn_repaint_thread(rx, result.clone(), cc.egui_ctx.clone());
        Self {
            pos: Position::L,
            var: Variable::AccelX,
            file_selects: Vec::new(),
            result,
            manager,
            state: State {
                show_side_panel: false,
                unprocess_dialog: false,
                show_process_win: false,
                show_boxplot: false,
                box_data: BoxData::MinMax,
                show_min: true,
                show_max: true,
                show_gait: true,
                show_db: true,
                show_lt: true,
                show_rt: true,
                show_lineplot: false,
                show_gait_line: true,
                show_contact: true,
            },
        }
    }

    pub fn open_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.state.show_process_win = true;
            self.manager.start_get_data_from_dir(path);
        }
    }

    pub fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.state.show_process_win = true;
            self.manager.start_get_data_from_file(path);
        }
    }

    pub fn close_all(&mut self) {
        self.manager.clear_message();
    }
}

impl super::View for Chart {
    fn show(&mut self, ctx: &eframe::egui::Context) {
        if self.state.show_side_panel {
            egui::SidePanel::right("right panel").show(ctx, |ui| {
                side_panel_ui(self, ui);
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.state.show_boxplot && !self.state.show_lineplot {
                let msg = if self.file_selects.len() == 0 {
                    "Please select one file"
                } else {
                    "Please select one chart"
                };
                ui.centered_and_justified(|ui| {
                    ui.label(msg);
                });
            }
            let egui::Vec2 { x, y } = ui.available_size();
            let size = egui::vec2(
                x,
                if self.state.show_boxplot && self.state.show_lineplot {
                    y / 2.
                } else {
                    y
                },
            );
            if self.state.show_boxplot {
                ui.allocate_ui(size, |ui| {
                    box_plot(self, ui);
                });
            }
            if self.state.show_lineplot {
                ui.allocate_ui(size, |ui| {
                    line_plot(self, ui);
                });
            }
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

        if self.state.show_process_win {
            egui::Window::new("process win")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                .title_bar(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Preprocessing");
                        ui.separator();
                    });
                    match &*self.result.lock().unwrap() {
                        Message::Running(progress, msg) => {
                            ui.add(
                                egui::ProgressBar::new(*progress).animate(true),
                            );
                            ui.label(format!("Processing: {}", msg));
                            ui.vertical_centered(|ui| {
                                if ui.button("Cancel").clicked() {
                                    self.manager.stop();
                                }
                            });
                        }
                        Message::Abort(msg) => {
                            ui.label(format!("Abort: {}", msg));
                        }
                        Message::Done(v) => {
                            if v.len() > 0 {
                                self.file_selects = vec![false; v.len()];
                                self.file_selects[0] = true;
                            }
                            self.state.show_boxplot = true;
                            self.state.show_lineplot = true;
                            self.state.show_side_panel = true;
                            self.state.show_process_win = false;
                        }
                        Message::Nothing => {}
                    }
                });
        }
    }
}

fn box_plot(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        file_selects,
        result,
        pos,
        var,
        state:
            State {
                box_data,
                show_min,
                show_max,
                show_gait,
                show_db,
                show_lt,
                show_rt,
                ..
            },
        ..
    } = app;
    let result = &*result.lock().unwrap();
    let v = match result {
        Message::Done(v) => v,
        _ => {
            return;
        }
    };
    Plot::new("Box Plot")
        .legend(
            Legend::default()
                .position(egui::plot::Corner::RightBottom)
                .text_style(egui::TextStyle::Small),
        )
        .show(ui, |plot_ui| {
            let mut i = 0.;
            for (f, selected) in v.into_iter().zip(file_selects.into_iter()) {
                if !*selected {
                    continue;
                }
                let (_, min_q, max_q) =
                    &f.raw.y.get(pos).unwrap().get(var).unwrap();
                let min_t = min_q.to_tuple();
                let max_t = max_q.to_tuple();
                let mut box_elems = Vec::new();
                match box_data {
                    BoxData::MinMax => {
                        if *show_min {
                            box_elems.push(
                                BoxElem::new(
                                    i,
                                    BoxSpread::new(
                                        min_t.0, min_t.1, min_t.2, min_t.3,
                                        min_t.4,
                                    ),
                                )
                                .box_width(0.1)
                                .whisker_width(0.1)
                                .name("Min Quantile"),
                            )
                        }
                        if *show_max {
                            box_elems.push(
                                BoxElem::new(
                                    i,
                                    BoxSpread::new(
                                        max_t.0, max_t.1, max_t.2, max_t.3,
                                        max_t.4,
                                    ),
                                )
                                .box_width(0.1)
                                .whisker_width(0.1)
                                .name("Max Quantile"),
                            )
                        }
                    }
                    BoxData::Support => {
                        if *show_gait {
                            let gait = &f.raw.gait.1.to_tuple();
                            box_elems.push(
                                BoxElem::new(
                                    i,
                                    BoxSpread::new(
                                        gait.0, gait.1, gait.2, gait.3, gait.4,
                                    ),
                                )
                                .box_width(0.1)
                                .whisker_width(0.1)
                                .name("Gait Quantile"),
                            );
                        }
                        if *show_db {
                            let db = &f.raw.db.to_tuple();
                            box_elems.push(
                                BoxElem::new(
                                    i + 0.3,
                                    BoxSpread::new(
                                        db.0, db.1, db.2, db.3, db.4,
                                    ),
                                )
                                .box_width(0.1)
                                .whisker_width(0.1)
                                .name("Double Support Quantile"),
                            );
                        }
                        if *show_lt {
                            let lt = &f.raw.lt.to_tuple();
                            box_elems.push(
                                BoxElem::new(
                                    i + 0.6,
                                    BoxSpread::new(
                                        lt.0, lt.1, lt.2, lt.3, lt.4,
                                    ),
                                )
                                .box_width(0.1)
                                .whisker_width(0.1)
                                .name("Left Single Support Quantile"),
                            );
                        }
                        if *show_rt {
                            let rt = &f.raw.rt.to_tuple();
                            box_elems.push(
                                BoxElem::new(
                                    i + 0.9,
                                    BoxSpread::new(
                                        rt.0, rt.1, rt.2, rt.3, rt.4,
                                    ),
                                )
                                .box_width(0.1)
                                .whisker_width(0.1)
                                .name("Right Single Support Quantile"),
                            );
                        }
                    }
                }
                plot_ui.box_plot(
                    BoxPlot::new(box_elems).horizontal().name(&*f.path),
                );
                match box_data {
                    BoxData::MinMax => i += 0.1,
                    BoxData::Support => i += 0.1,
                };
            }
        });
}
fn line_plot(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        file_selects,
        pos,
        var,
        result,
        state:
            State {
                show_gait_line,
                show_contact,
                ..
            },
        ..
    } = app;
    let result = &*result.lock().unwrap();
    let v = match result {
        Message::Done(v) => v,
        _ => {
            return;
        }
    };
    Plot::new("Line Plot")
        .legend(
            Legend::default()
                .position(egui::plot::Corner::RightBottom)
                .text_style(egui::TextStyle::Small),
        )
        .show(ui, |plot_ui| {
            for (f, selected) in v.into_iter().zip(file_selects.into_iter()) {
                if !*selected {
                    continue;
                }
                let (x, (y, min_q, max_q)) =
                    (&f.raw.x, &f.raw.y.get(pos).unwrap().get(var).unwrap());
                if *show_gait_line {
                    for value in f.raw.gait.0.iter() {
                        plot_ui.vline(
                            VLine::new(*value)
                                .color(egui::Color32::GRAY)
                                .style(egui::plot::LineStyle::dotted_dense()),
                        );
                    }
                }
                if *show_contact {
                    let line = Line::new(
                        x.into_iter()
                            .zip((&f.raw.l_contact).into_iter())
                            .map(|(a, b)| {
                                let upper_bound = if b.gt(&0) {
                                    max_q.max()
                                } else {
                                    min_q.min()
                                };
                                [a.clone(), upper_bound]
                            })
                            .collect::<PlotPoints>(),
                    )
                    .fill(min_q.min() as f32)
                    .color(egui::Color32::LIGHT_BLUE)
                    .width(0.)
                    .name("L Contact");
                    plot_ui.line(line);
                    let line = Line::new(
                        x.into_iter()
                            .zip((&f.raw.r_contact).into_iter())
                            .map(|(a, b)| {
                                let upper_bound = if b.gt(&0) {
                                    max_q.max()
                                } else {
                                    min_q.min()
                                };
                                [a.clone(), upper_bound]
                            })
                            .collect::<PlotPoints>(),
                    )
                    .fill(min_q.min() as f32)
                    .color(egui::Color32::LIGHT_GREEN)
                    .width(0.)
                    .name("R Contact");
                    plot_ui.line(line);
                }
                let data: PlotPoints = x
                    .into_iter()
                    .zip(y.into_iter())
                    .map(|(a, b)| [a.clone(), b.clone()])
                    .collect();
                plot_ui.line(Line::new(data).name(&*f.path));
            }
        });
}

fn side_panel_ui(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        result,
        file_selects,
        pos,
        var,
        state:
            State {
                show_boxplot,
                show_lineplot,
                box_data,
                show_min,
                show_max,
                show_gait,
                show_db,
                show_lt,
                show_rt,
                show_gait_line,
                show_contact,
                ..
            },
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
        });
    ui.separator();
    egui::CollapsingHeader::new("Box plot")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Enable");
                ui.checkbox(show_boxplot, "");
            });
            ui.horizontal(|ui| {
                ui.label("Data");
                ui.radio_value(box_data, BoxData::MinMax, "min max");
                ui.radio_value(box_data, BoxData::Support, "support");
            });
            match box_data {
                BoxData::MinMax => {
                    ui.label("Options");
                    ui.horizontal(|ui| {
                        ui.toggle_value(show_min, "min");
                        ui.toggle_value(show_max, "max");
                    });
                }
                BoxData::Support => {
                    ui.label("Options");
                    ui.horizontal(|ui| {
                        ui.toggle_value(show_gait, "Gait");
                        ui.toggle_value(show_db, "DB");
                        ui.toggle_value(show_lt, "LT");
                        ui.toggle_value(show_rt, "RT");
                    });
                }
            }
        });
    egui::CollapsingHeader::new("Line plot")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Enable");
                ui.checkbox(show_lineplot, "");
            });
            ui.horizontal(|ui| {
                ui.label("Gait line");
                ui.checkbox(show_gait_line, "");
            });
            ui.horizontal(|ui| {
                ui.label("Support");
                ui.checkbox(show_contact, "");
            });
        });
    ui.separator();
    ui.heading("File Lists");
    ui.group(|ui| {
        ScrollArea::horizontal()
            .vscroll(true)
            // .max_height(200.0)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Grid::new("files").striped(true).num_columns(1).show(
                    ui,
                    |ui| {
                        let result = &*result.lock().unwrap();
                        let v = match result {
                            Message::Done(v) => v,
                            _ => {
                                return;
                            }
                        };
                        for (DataInfo { path, .. }, selected) in
                            v.into_iter().zip(file_selects.into_iter())
                        {
                            ui.checkbox(selected, path.as_str());
                            ui.end_row();
                        }
                    },
                );
            });
    });
}

fn spawn_repaint_thread<T: std::marker::Send + 'static>(
    rx: std::sync::mpsc::Receiver<T>,
    message: Arc<Mutex<T>>,
    ctx: egui::Context,
) {
    std::thread::spawn(move || loop {
        if let Ok(recv_message) = rx.recv() {
            *message.lock().unwrap() = recv_message;
            ctx.request_repaint();
        }
    });
}
