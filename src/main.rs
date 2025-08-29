use std::{
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    thread::JoinHandle,
    time::Duration
};
use eframe::egui;
use notify_rust::{Notification, Hint};


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
    app_state: State,
    run_state: RunState,
    running: bool,
    pause: bool,
    time_sec: Arc<Mutex<u32>>,
    cur_lap: u8,
    cur_loop: u8,
    lap_dur_min: u32,
    rest_lap_min: u32,
    rest_loop_min: u32,
    pause_flag: Arc<AtomicBool>,
    thread_done_flag: Arc<AtomicBool>,
    child_process: Option<JoinHandle<()>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            app_state: State::STEADY,
            run_state: RunState::LAP,
            running: false, 
            pause: true,
            time_sec: Arc::new(Mutex::new(25*60)), 
            cur_lap: 0, 
            cur_loop: 0, 
            lap_dur_min: 25,
            rest_lap_min: 5, 
            rest_loop_min: 30,
            pause_flag: Arc::new(AtomicBool::new(false)),
            thread_done_flag: Arc::new(AtomicBool::new(false)),
            child_process: None,
        }
    }
}

impl MyApp {
    fn steady(&mut self, ui: &mut egui::Ui) {
        let time = *self.time_sec.lock().unwrap();
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let duration_time = format!("{:02}:{:02}", time/60, time%60);
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

                    let thread_time = Arc::clone(&self.time_sec);
                    let thread_done_flag = Arc::clone(&self.thread_done_flag);
                    let pause_flag = Arc::clone(&self.pause_flag);

                    self.pause_flag.store(false, Ordering::Relaxed);

                    let child = thread::spawn(move || {
                        loop {
                            if pause_flag.load(Ordering::Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_secs(1));
                            let mut t = thread_time.lock().unwrap();
                            if *t > 0 {
                                *t -= 1;
                            }
                        }
                        thread_done_flag.store(true, Ordering::Relaxed);
                    });

                    self.child_process = Some(child);
                }
            } else {
                if ui.button(egui::RichText::new("⏸").font(egui::FontId::proportional(30.0))).clicked() {
                    self.pause = true; 

                    self.pause_flag.store(true, Ordering::Relaxed);
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
                ui.add(egui::DragValue::new(&mut self.lap_dur_min).range(1..=59));
                ui.label("minutes");
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("Lap rest duration: ");
                ui.add(egui::DragValue::new(&mut self.rest_lap_min).range(1..=59).speed(1));
                ui.label("minutes");
            });
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.label("Loop rest duration: ");
                ui.add(egui::DragValue::new(&mut self.rest_loop_min).range(1..=59).speed(1));
                ui.label("minutes");
            });
            ui.add_space(25.0);
            if ui.button("confirm").clicked() {
                self.time_sec = Arc::new(Mutex::new(self.lap_dur_min * 60));
                self.app_state = State::STEADY;
            }
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(500));
        let time = *self.time_sec.lock().unwrap();
        if self.running && !self.pause {
            if time <= 0 && self.run_state == RunState::LAP {
                self.pause = true;
                let _ = Notification::new()
                    .summary("Pomodoro")
                    .body("Time out! Please check your tomato!")
                    .appname("pomodoro")
                    .hint(Hint::Resident(true))
                    .timeout(0)
                    .show();
                self.cur_lap += 1;
                if self.cur_lap > 3 {
                    self.time_sec = Arc::new(Mutex::new(self.rest_loop_min*60));
                    self.run_state = RunState::RestLoop;
                    self.cur_lap = 0;
                    self.cur_loop += 1;
                } else {
                    self.time_sec = Arc::new(Mutex::new(self.rest_lap_min*60));
                    self.run_state = RunState::RestLap;
                }
            } else if time <= 0 && self.run_state != RunState::LAP {
                self.time_sec = Arc::new(Mutex::new(self.lap_dur_min * 60));
                self.run_state = RunState::LAP;
            } 
        }
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            match self.app_state {
                State::STEADY => self.steady(ui),
                State::SETTING => self.setting(ui),
            }
        });

        // ✅ Check if background thread ended and clean up
        if self.thread_done_flag.load(Ordering::Relaxed) {
            if let Some(child) = self.child_process.take() {
                let _ = child.join(); // safe: thread already exited
            }
            self.thread_done_flag.store(false, Ordering::Relaxed); // reset
        }
    }
}
