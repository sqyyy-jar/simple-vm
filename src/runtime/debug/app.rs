use std::time::Duration;

use eframe::egui;

use super::Debugger;

const ICON_RESUME: egui::ImageSource = egui::include_image!("../../../assets/icons/resume.png");
const ICON_PAUSE: egui::ImageSource = egui::include_image!("../../../assets/icons/pause.png");
const ICON_STEP: egui::ImageSource = egui::include_image!("../../../assets/icons/step.png");

pub struct DebugApp {
    debugger: Debugger,
    selected_frame: usize,
    use_last_frame: bool,
}

impl DebugApp {
    pub fn run(debugger: Debugger) {
        let app = Box::new(DebugApp {
            debugger,
            selected_frame: 0,
            use_last_frame: true,
        });
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Debugger",
            options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                app
            }),
        )
        .unwrap();
    }
}

impl eframe::App for DebugApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.debugger.finished && !self.debugger.paused {
            self.debugger
                .resume_with_timeout(true, Duration::from_millis(10));
        }
        egui::TopBottomPanel::top("top")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let pause_btn = if self.debugger.paused || self.debugger.finished {
                        egui::Button::image_and_text(egui::Image::new(ICON_RESUME), "Resume")
                    } else {
                        egui::Button::image_and_text(egui::Image::new(ICON_PAUSE), "Pause")
                    };
                    if ui.add(pause_btn).clicked() {
                        self.debugger.paused = !self.debugger.paused;
                    }
                    if ui
                        .add(egui::Button::image_and_text(
                            egui::Image::new(ICON_STEP),
                            "Step",
                        ))
                        .clicked()
                        && !self.debugger.finished
                        && self.debugger.paused
                    {
                        self.debugger.step();
                    }
                });
            });
        if self.use_last_frame {
            self.selected_frame = self.debugger.callstack.len() - 1;
        }
        egui::SidePanel::left("left")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Debugger");
                for (i, _frame) in self.debugger.callstack.iter().enumerate().rev() {
                    if ui
                        .selectable_label(i == self.selected_frame, format!("Frame {i}"))
                        .clicked()
                    {
                        self.selected_frame = i;
                        self.use_last_frame = i == self.debugger.callstack.len() - 1;
                    }
                }
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.code(format!("pc: {:?}", self.debugger.runtime.pc));
            let Some(frame) = self.debugger.callstack.get(self.selected_frame) else {
                return;
            };
            ui.code(format!(
                "proc: {:?}
fp: {:?}
size: {}",
                frame.proc, frame.fp, frame.size
            ));
            for offset in 0..frame.size {
                unsafe {
                    let value = *frame.fp.sub(1 + offset);
                    let proc = value.proc;
                    let s64 = value.s64;
                    ui.code(format!("  [{offset}]: {proc:012?} ~ {s64}"));
                }
            }
        });
    }
}
