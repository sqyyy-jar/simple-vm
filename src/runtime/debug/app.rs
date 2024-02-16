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
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 600.0])
                .with_min_inner_size([500.0, 300.0]),
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

    fn draw_top(&mut self, ui: &mut egui::Ui) {
        ui.set_enabled(!self.debugger.finished);
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
                && self.debugger.paused
            {
                self.debugger.step();
            }
        });
    }

    fn draw_side_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Stack");
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let spacing = &mut ui.spacing_mut().scroll;
                spacing.floating = false;
                // ui.avail
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
    }

    fn draw_central_panel(&mut self, ui: &mut egui::Ui) {
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
    }
}

impl eframe::App for DebugApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.debugger.finished && !self.debugger.paused {
            self.debugger
                .resume_with_timeout(true, Duration::from_millis(10));
        }
        ctx.style_mut(|style| {
            style.spacing.scroll = egui::style::ScrollStyle::solid();
        });
        egui::TopBottomPanel::top("top")
            .resizable(false)
            .show(ctx, |ui| self.draw_top(ui));
        if self.use_last_frame || self.selected_frame >= self.debugger.callstack.len() {
            self.selected_frame = self.debugger.callstack.len() - 1;
        }
        egui::SidePanel::left("left")
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(20, 20, 20)))
            .resizable(true)
            .show(ctx, |ui| self.draw_side_panel(ui));
        egui::CentralPanel::default().show(ctx, |ui| self.draw_central_panel(ui));
    }
}
