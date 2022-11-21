use std::sync::{Arc, Mutex};

use eframe::egui::{
    self,
    plot::{BoxElem, BoxPlot, BoxSpread, Legend, Line, Plot, PlotPoints},
    ScrollArea,
};

use crate::{
    config::{Position, Variable},
    data_process::{DataInfo, Manager, Message},
};

pub struct State {
    pub show_side_panel: bool,
    pub unprocess_dialog: bool,
    show_process_win: bool,
    show_boxplot: bool,
    show_lineplot: bool,
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
                show_lineplot: false,
            },
        }
    }

    pub fn open_dir(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.state.show_process_win = true;
            self.manager.start_get_data(path);
        }
    }

    pub fn close_dir(&mut self) {
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
                ui.centered_and_justified(|ui| {
                    ui.label("Please select one chart");
                });
            }
            let egui::Vec2 { x, y } = ui.available_size();
            let size = egui::vec2(x, y / 2.);
            if self.state.show_boxplot {
                egui::Resize::default()
                    .id_source("boxplot")
                    .default_size(size)
                    .max_size([x, y])
                    .show(ui, |ui| {
                        box_plot(self, ui);
                    });
            }
            if self.state.show_lineplot {
                egui::Resize::default()
                    .id_source("lineplot")
                    .default_size(size)
                    .max_size([x, y])
                    .show(ui, |ui| {
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
                    });
                    match &*self.result.lock().unwrap() {
                        Message::Running(progress, msg) => {
                            ui.label(format!("Processing: {}", msg));
                            ui.add(
                                egui::ProgressBar::new(*progress).animate(true),
                            );
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
        .include_x(0.)
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
                plot_ui.box_plot(
                    BoxPlot::new(vec![
                        BoxElem::new(
                            i + 0.,
                            BoxSpread::new(
                                min_t.0, min_t.1, min_t.2, min_t.3, min_t.4,
                            ),
                        )
                        .box_width(0.1)
                        .whisker_width(0.1)
                        .name("min"),
                        BoxElem::new(
                            i + 1.,
                            BoxSpread::new(
                                max_t.0, max_t.1, max_t.2, max_t.3, max_t.4,
                            ),
                        )
                        .box_width(0.1)
                        .whisker_width(0.1)
                        .name("max"),
                    ])
                    .name(&*f.path),
                );
                i += 0.1;
            }
        });
}
fn line_plot(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        file_selects,
        pos,
        var,
        result,
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
        .include_x(0.)
        .include_y(0.)
        .show(ui, |plot_ui| {
            for (f, selected) in v.into_iter().zip(file_selects.into_iter()) {
                if !*selected {
                    continue;
                }
                let (x, (y, min_q, max_q)) =
                    (&f.raw.x, &f.raw.y.get(pos).unwrap().get(var).unwrap());
                plot_ui.line(
                    Line::new(
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
                    .name("L Contact"),
                );
                plot_ui.line(
                    Line::new(
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
                    .name("R Contact"),
                );
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
            ui.end_row();
            if ui.button("reset").clicked() {
                ui.ctx().memory().reset_areas();
            }
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
