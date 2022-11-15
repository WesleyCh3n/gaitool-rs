use eframe::egui::{
    self,
    plot::{BoxElem, BoxPlot, BoxSpread, Legend, Line, Plot, PlotPoints},
    ScrollArea,
};

pub struct State {
    pub side_panel_open: bool,
    unprocess_dialog: bool,
}

pub struct FileList {
    path: String,
    is_selected: bool,
}

pub struct Chart {
    pos_list: Vec<String>,
    pos_selected: Option<usize>,
    var_list: Vec<String>,
    var_selected: Option<usize>,
    file_list: Vec<FileList>,
    show_boxplot: bool,
    show_lineplot: bool,
    pub state: State,
}

impl Chart {
    pub fn new() -> Self {
        Self {
            pos_list: Vec::new(),
            pos_selected: None,
            var_list: Vec::new(),
            var_selected: None,
            file_list: Vec::new(),
            show_boxplot: false,
            show_lineplot: false,
            state: State {
                side_panel_open: false,
                unprocess_dialog: false,
            },
        }
    }

    pub fn open_dir<P: AsRef<std::path::Path>>(&mut self, _path: P) {
        let Self {
            pos_list: _,
            pos_selected: _,
            var_list: _,
            var_selected: _,
            file_list: _,
            ..
        } = self;
        // open folder
        // unprocess_vec = []
        // loop folder
        //   detect if files process
        //   if true
        //     pos_list.push(pos)
        //     var_list.push(var)
        //   else
        //     unprocess_vec.push(file)
        // if !unprocess_vec.is_empty()
        //   show_pop_up_win_flag = true

        // actually this should be a long task so it should be in a thread
        std::thread::spawn(|| {
            println!("test");
        });
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            for entries in std::fs::read_dir(path).unwrap() {
                if let Ok(entry) = entries {
                    // preprocess
                    let raw_file = std::fs::File::open(entry.path())
                        .expect("Can't open raw file");
                    let reader_raw = std::io::BufReader::new(raw_file);
                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(false)
                        .from_reader(reader_raw);
                    println!("{:?}", entry);
                    for result in rdr.records().take(2) {
                        println!("result: {:?}", result);
                    }
                    // file_list.push(FileList {
                    //     path: entry.file_name().to_str().unwrap().to_string(),
                    //     is_selected: false,
                    // })
                }
            }
        }
        // *pos_list = vec!["L", "T"].iter().map(|s| s.to_string()).collect();
        // *pos_selected = Some(0);
        // *var_list = vec!["Acceleration X", " Gyroscope X"]
        //     .iter()
        //     .map(|s| s.to_string())
        //     .collect();
        // *var_selected = Some(0);
        // file_list.push(FileList {
        //     path: String::from("this is file 1"),
        //     is_selected: true,
        // });
        // file_list.push(FileList {
        //     path: String::from("this is file 2"),
        //     is_selected: false,
        // });
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
        ..
    } = app;
    if !*show_boxplot && !*show_lineplot {
        ui.centered_and_justified(|ui| {
            ui.label("Please select one chart");
        });
    }
    let egui::Vec2 { y, .. } = ui.available_size();
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
            .height(if !*show_boxplot { y } else { y * 2. / 3. })
            .include_x(0.) // show x axis label
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new(sin).name("sin"));
                plot_ui.line(Line::new(cos).name("cos"));
            });
    }
}

fn side_panel_ui(app: &mut Chart, ui: &mut eframe::egui::Ui) {
    let Chart {
        show_boxplot,
        show_lineplot,
        pos_list,
        pos_selected,
        var_list,
        var_selected,
        file_list,
        ..
    } = app;
    egui::Grid::new("options")
        .max_col_width(100.)
        .striped(true)
        .show(ui, |ui| {
            ui.heading("Options");
            ui.end_row();
            ui.label("Position: ");
            let selected_text = if let Some(i) = pos_selected {
                pos_list[*i].clone()
            } else {
                "".to_owned()
            };
            egui::ComboBox::from_id_source("combo")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    pos_list.iter().enumerate().for_each(|(i, p)| {
                        ui.selectable_value(pos_selected, Some(i), p);
                    });
                });
            ui.end_row();
            ui.label("Variable: ");
            let selected_text = if let Some(i) = var_selected {
                var_list[*i].clone()
            } else {
                "".to_owned()
            };
            egui::ComboBox::from_id_source("var_combo")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    var_list.iter().enumerate().for_each(|(i, v)| {
                        ui.selectable_value(var_selected, Some(i), v);
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
