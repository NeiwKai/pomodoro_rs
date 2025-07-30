use std::time::{Duration, Instant};
use eframe::egui;


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Pomodoro",
        options,
        Box::new(|_cc: &eframe::CreationContext<'_>| {
            Ok(Box::new(MyApp::default()))
        })
    )
}

enum State {
    STEADY,
    SETTING,
}

#[derive(PartialEq)]
enum RunState {
    LAP,
    RestLap,
    RestLoop,
}

struct MyApp {
    last_update: Instant,
    app_state: State,
    run_state: RunState,
    running: bool,
    pause: bool,
    time_sec: f32,
    cur_lap: u8,
    cur_loop: u8,
    lap_dur_min: f32,
    rest_lap_min: f32,
    rest_loop_min: f32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            app_state: State::STEADY,
            run_state: RunState::LAP,
            running: false, 
            pause: true,
            time_sec: 25.0*60.0, 
            cur_lap: 0, 
            cur_loop: 0, 
            lap_dur_min: 25.0,
            rest_lap_min: 5.0, 
            rest_loop_min: 30.0,
        }
    }
}

impl MyApp {
    fn steady(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let duration_time = format!("{:02}:{:02}", (self.time_sec%3600.0)/60.0, self.time_sec%60.0);
            match self.run_state {
                RunState::LAP => ui.label(egui::RichText::new("grinding...").font(egui::FontId::proportional(10.0))),
                RunState::RestLap => ui.label(egui::RichText::new("lap resting...").font(egui::FontId::proportional(10.0))),
                RunState::RestLoop => ui.label(egui::RichText::new("loop resting...").font(egui::FontId::proportional(10.0))),
            };
            ui.label(egui::RichText::new(format!("{}", duration_time)).font(egui::FontId::proportional(100.0)));
            ui.label(egui::RichText::new(format!("Lap: {}/4, Loop {}", self.cur_lap, self.cur_loop)).font(egui::FontId::proportional(20.0)));
            ui.add_space(100.0);
            if self.pause {
                if ui.button(egui::RichText::new("▶").font(egui::FontId::proportional(30.0))).clicked() {
                    self.pause = false; 
                    self.running = true;
                }
            } else {
                if ui.button(egui::RichText::new("⏸").font(egui::FontId::proportional(30.0))).clicked() {
                    self.pause = true; 
                }
            } 
            ui.add_space(20.0);
            if !self.running {
                if ui.button(egui::RichText::new("⚙").font(egui::FontId::proportional(30.0))).clicked() {
                    self.app_state = State::SETTING;
                }
            } else if self.pause {
                if ui.button(egui::RichText::new("⏹").font(egui::FontId::proportional(30.0))).clicked() {
                    self.running = false;
                    *self = MyApp::default();
                }
            } 
        });
    }
    fn setting(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label("setting");
            ui.add_space(50.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("Lap duration: ");
                ui.add(egui::DragValue::new(&mut self.lap_dur_min).range(1.0..=59.0));
                ui.label("minutes");
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("Lap rest duration: ");
                ui.add(egui::DragValue::new(&mut self.rest_lap_min).range(1.0..=59.0).speed(1.0));
                ui.label("minutes");
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("Loop rest duration: ");
                ui.add(egui::DragValue::new(&mut self.rest_loop_min).range(1.0..=59.0).speed(1.0));
                ui.label("minutes");
            });
            ui.add_space(25.0);
            if ui.button("confirm").clicked() {
                self.time_sec = self.lap_dur_min * 60.0;
                self.app_state = State::STEADY;
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running && !self.pause {
            ctx.request_repaint();
            if self.last_update.elapsed() >= Duration::from_secs(1) {
                self.last_update = Instant::now();
                self.time_sec -= 1.0;

                if self.time_sec <= 0.0 && self.run_state == RunState::LAP {
                    self.cur_lap += 1;
                    if self.cur_lap > 3 {
                        self.time_sec = self.rest_loop_min*60.0;
                        self.run_state = RunState::RestLoop;
                        self.cur_lap = 0;
                        self.cur_loop += 1;
                    } else {
                        self.time_sec = self.rest_lap_min*60.0;
                        self.run_state = RunState::RestLap;
                    }
                } else if self.time_sec <= 0.0 && self.run_state != RunState::LAP {
                    self.time_sec = self.lap_dur_min * 60.0;
                    self.run_state = RunState::LAP;
                } 
            }
        }
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            match self.app_state {
                State::STEADY => self.steady(ui),
                State::SETTING => self.setting(ui),
            }
        });
    }
}
