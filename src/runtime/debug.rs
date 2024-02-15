use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use eframe::egui;

use crate::{
    opcodes::{
        Alloc, Call, CallDynamic, Instruction, ALLOC, BREAKPOINT, CALL, CALL_DYNAMIC, RETURN,
    },
    value::Value,
};

use super::{proc::Proc, Runtime};

pub struct Debugger {
    runtime: Runtime,
    breakpoints: HashMap<*const u8, Breakpoint>,
    callstack: Vec<CallFrameInfo>,
    /// Toggle for the debugger app
    paused: bool,
    finished: bool,
}

impl Debugger {
    pub fn new(mut runtime: Runtime, main: u32) -> Self {
        runtime.call(main);
        let main = &*runtime.procs[main as usize];
        let callstack = vec![CallFrameInfo::new(main, runtime.stack.fp)];
        Self {
            runtime,
            breakpoints: HashMap::new(),
            callstack,
            paused: true,
            finished: false,
        }
    }

    pub fn add_breakpoint(&mut self, proc_index: u32, offset: usize) {
        let Some(proc) = self.runtime.procs.get(proc_index as usize) else {
            return;
        };
        let address = &proc.code[offset];
        self.breakpoints.insert(address, Breakpoint::new(proc));
    }

    pub fn execute(&mut self, opcode: u8) {
        debug_assert!(!self.finished);
        match opcode {
            ALLOC => {
                let insn = Alloc::read(&mut self.runtime);
                self.runtime.stack.alloc(insn.size as usize);
                // Track frame size
                self.callstack.last_mut().unwrap().size += insn.size as usize;
            }
            CALL => {
                let insn = Call::read(&mut self.runtime);
                self.runtime.call(insn.index);
                // Track callframe
                self.callstack.push(CallFrameInfo::new(
                    &*self.runtime.procs[insn.index as usize],
                    self.runtime.stack.fp,
                ));
            }
            CALL_DYNAMIC => {
                let insn = CallDynamic::read(&mut self.runtime);
                let proc = unsafe { self.runtime.stack.load(insn.src).proc };
                unsafe { self.runtime.push_call_frame(proc) };
                // Track callframe
                self.callstack
                    .push(CallFrameInfo::new(proc, self.runtime.stack.fp));
            }
            RETURN => {
                let ra = self.runtime.stack.return_call();
                self.runtime.pc = ra;
                // Untrack callframe
                self.callstack.pop().unwrap();
            }
            BREAKPOINT => {
                self.paused = true;
            }
            _ => self.runtime.execute(opcode),
        }
    }

    pub fn step(&mut self) {
        debug_assert!(!self.finished);
        let opcode = self.runtime.fetch();
        self.execute(opcode);
    }

    pub fn resume(&mut self, skip_first: bool) -> Option<Breakpoint> {
        debug_assert!(!self.finished);
        if skip_first {
            self.step();
        }
        while !self.paused && !self.runtime.pc.is_null() {
            if let Some(breakpoint) = self.breakpoints.get(&self.runtime.pc) {
                self.paused = true;
                return Some(breakpoint.clone());
            }
            self.step()
        }
        self.finished = true;
        None
    }

    pub fn resume_with_timeout(
        &mut self,
        skip_first: bool,
        timeout: Duration,
    ) -> Option<Breakpoint> {
        debug_assert!(!self.finished);
        let start = Instant::now();
        if skip_first {
            self.step();
        }
        loop {
            /// Iterations between checks
            const ITERS_PER_CHECK: usize = 255;
            for _ in 0..ITERS_PER_CHECK {
                if self.paused {
                    return None;
                }
                if self.runtime.pc.is_null() {
                    self.finished = true;
                    return None;
                }
                if let Some(breakpoint) = self.breakpoints.get(&self.runtime.pc) {
                    self.paused = true;
                    return Some(breakpoint.clone());
                }
                self.step();
            }
            if Instant::now().duration_since(start) >= timeout {
                return None;
            }
        }
    }

    pub fn start_app(self) {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 375.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Debugger",
            options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);

                Box::new(self)
            }),
        )
        .unwrap();
    }
}

impl eframe::App for Debugger {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.finished && !self.paused {
            self.resume_with_timeout(true, Duration::from_millis(10));
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Debugger");
            ui.horizontal(|ui| {
                let pause_btn = if self.paused {
                    egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../../assets/icons/resume.png")),
                        "Resume",
                    )
                } else {
                    egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../../assets/icons/pause.png")),
                        "Pause",
                    )
                };
                if ui.add(pause_btn).clicked() {
                    self.paused = !self.paused;
                }
                if ui
                    .add(egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../../assets/icons/step.png")),
                        "Step",
                    ))
                    .clicked()
                    && !self.finished
                    && self.paused
                {
                    self.step();
                }
            });
            let mut debug_info = format!("pc: {:?}", self.runtime.pc,);
            for i in 0..self.callstack.len() {
                let (proc, fp, frame_size) = {
                    let info = &self.callstack[i];
                    (info.proc, info.fp, info.size)
                };
                debug_info.push_str(&format!(
                    "
frame[{i}]:
  proc: {proc:?}
  fp: {fp:?}
  size: {frame_size}"
                ));
                for offset in 0..frame_size {
                    unsafe {
                        let value = *fp.sub(1 + offset);
                        let proc = value.proc;
                        let s64 = value.s64;
                        debug_info.push_str(&format!("\n  [{offset}]: {proc:012?} ~ {s64}"));
                    }
                }
            }
            ui.code_editor(&mut debug_info);
        });
    }
}

#[derive(Clone)]
pub struct Breakpoint {
    pub proc: *const Proc,
}

impl Breakpoint {
    pub fn new(proc: &Proc) -> Self {
        Self { proc }
    }
}

pub struct CallFrameInfo {
    pub proc: *const Proc,
    pub fp: *const Value,
    pub size: usize,
}

impl CallFrameInfo {
    pub fn new(proc: *const Proc, fp: *const Value) -> Self {
        Self { proc, fp, size: 0 }
    }
}
