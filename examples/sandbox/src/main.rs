use std::fmt::Display;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

use eframe::egui;

use egui_modal_spinner::ModalSpinner;

#[derive(PartialEq)]
enum ThreadState {
    LoadingA,
    LoadingB,
    LoadingC,
    Finished,
}

impl Display for ThreadState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoadingA => write!(f, "Loading dogs 🐕 ..."),
            Self::LoadingB => write!(f, "Loading cats 🐈 ..."),
            Self::LoadingC => write!(f, "Loading penguins 🐧 ..."),
            Self::Finished => write!(f, "Finished"),
        }
    }
}

struct MyApp {
    spinner: ModalSpinner,
    result_recv: Option<mpsc::Receiver<ThreadState>>,
    thread_state: Option<ThreadState>,
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            spinner: ModalSpinner::new(),
            result_recv: None,
            thread_state: None,
        }
    }

    fn exec_task(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.result_recv = Some(rx);
        self.thread_state = None;

        thread::spawn(move || {
            let _ = tx.send(ThreadState::LoadingA);
            thread::sleep(std::time::Duration::from_secs(2));

            let _ = tx.send(ThreadState::LoadingB);
            thread::sleep(std::time::Duration::from_secs(1));

            let _ = tx.send(ThreadState::LoadingC);
            thread::sleep(std::time::Duration::from_secs(2));

            let _ = tx.send(ThreadState::Finished);
        });
    }

    fn update_task_thread(&mut self) {
        if let Some(rx) = &self.result_recv {
            match rx.try_recv() {
                Ok(state) => {
                    if state == ThreadState::Finished {
                        self.spinner.close();
                        self.result_recv = None;
                        self.thread_state = None;
                    }

                    self.thread_state = Some(state);
                }
                Err(err) => {
                    if err == TryRecvError::Disconnected {
                        self.spinner.close();
                        self.result_recv = None;
                        println!("thread ended unexpectedly");
                    }
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.style_mut().animation_time = 0.1;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("My egui application");
            egui::widgets::global_theme_preference_buttons(ui);

            if ui.button("Do something resource heavy!").clicked() {
                self.exec_task();

                self.spinner.open();
            }

            self.update_task_thread();

            self.spinner.update_with_content(ui, |ui| {
                if let Some(s) = &self.thread_state {
                    ui.add_space(ui.spacing().item_spacing.y);
                    ui.label(s.to_string());
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1080.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui application",
        options,
        Box::new(|_| Ok(Box::new(MyApp::new()))),
    )
}
