pub const UNIT: Value = Value { as_unit: () };

#[macro_export]
macro_rules! proc {
    ($(:$insn: expr)*) => {
        {
            use $crate::rt::Insn::*;
            $crate::rt::Proc {
                code: vec![
                    $($insn),*
                ],
            }
        }
    };
}

#[macro_export]
macro_rules! make_runtime {
    (
        let constants = [ $($constant: expr),* ];
        let procs = [ $($proc_name: ident = { $($insn: expr;)* }),* ];
    ) => {{
        let mut runtime = $crate::rt::Runtime::new();
        $(
            runtime.push_constant(std::convert::Into::into($constant));
        )*
        $({
            #[allow(unused_imports)]
            use $crate::rt::_proc::*;
            runtime.push_constant(std::convert::Into::into($crate::rt::Proc {
                code: vec![
                    $($insn),*
                ],
            }));
        })*
        runtime
    }};
}

pub struct Runtime {
    pub constants: Vec<Constant>,
    pub frames: Vec<StackFrame>,
    pub stack: Vec<Value>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            frames: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn push_constant(&mut self, constant: Constant) {
        self.constants.push(constant);
    }

    pub fn push_call_frame(&mut self, index: u32) {
        let Constant::Proc(proc) = &self.constants[index as usize] else {
            panic!("Unable to call constants[{index}]");
        };
        self.frames.push(StackFrame {
            code: proc.code.as_slice(),
            offset: 0,
        });
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    #[inline]
    pub fn pop2(&mut self) -> (Value, Value) {
        (self.pop(), self.pop())
    }

    #[inline]
    pub fn pop_i64(&mut self) -> i64 {
        unsafe { self.pop().as_i64 }
    }

    #[inline]
    pub fn pop2_i64(&mut self) -> (i64, i64) {
        (self.pop_i64(), self.pop_i64())
    }

    #[inline]
    pub fn pop_f64(&mut self) -> f64 {
        unsafe { self.pop().as_f64 }
    }

    #[inline]
    pub fn pop2_f64(&mut self) -> (f64, f64) {
        (self.pop_f64(), self.pop_f64())
    }

    #[inline]
    pub fn pop_proc(&mut self) -> *const Proc {
        unsafe { self.pop().as_proc }
    }

    #[inline]
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline]
    pub fn push_i64(&mut self, value: i64) {
        self.stack.push(Value { as_i64: value });
    }

    #[inline]
    pub fn push_f64(&mut self, value: f64) {
        self.stack.push(Value { as_f64: value });
    }

    pub fn stack_alloc(&mut self, capacity: u32) {
        self.stack.reserve(capacity as usize);
        for _ in 0..capacity {
            self.push(UNIT);
        }
    }

    #[inline]
    pub fn copy(&mut self) {
        let top = *self.stack.last().unwrap();
        self.push(top);
    }

    #[inline]
    pub fn swap(&mut self) {
        let (top, bottom) = self.pop2();
        self.push(top);
        self.push(bottom);
    }

    #[inline]
    pub fn load(&mut self, offset: u32) {
        let index = self.stack.len() - offset as usize - 1;
        self.push(self.stack[index]);
    }

    #[inline]
    pub fn store(&mut self, offset: u32) {
        let value = self.pop();
        let index = self.stack.len() - offset as usize - 1;
        self.stack[index] = value;
    }

    pub fn load_const(&mut self, index: u32) {
        let constant = &self.constants[index as usize];
        self.push(match constant {
            Constant::I64(value) => Value { as_i64: *value },
            Constant::F64(value) => Value { as_f64: *value },
            Constant::Proc(value) => Value {
                as_proc: value.as_ref(),
            },
        });
    }

    pub fn call(&mut self) {
        let proc = self.pop_proc();
        self.frames.push(StackFrame {
            code: unsafe { (*proc).code.as_slice() },
            offset: 0,
        });
    }

    pub fn call_immediate(&mut self, proc: *const Proc) {
        self.frames.push(StackFrame {
            code: unsafe { (*proc).code.as_slice() },
            offset: 0,
        });
    }

    #[inline]
    pub fn jump(&mut self, offset: i32) {
        let frame = self.frames.last_mut().unwrap();
        frame.offset = frame.offset.wrapping_add_signed(offset as isize);
    }

    #[inline]
    pub fn inc_pc(&mut self) {
        self.frames.last_mut().unwrap().offset += 1;
    }

    pub fn evaluate(&mut self, insn: Insn) {
        match insn {
            Insn::StackAlloc(cap) => self.stack_alloc(cap),
            Insn::Copy => self.copy(),
            Insn::Swap => self.swap(),
            Insn::Load(offset) => self.load(offset),
            Insn::Store(offset) => self.store(offset),
            Insn::LoadConst(index) => self.load_const(index),
            Insn::Call => {
                self.inc_pc();
                self.call();
                return;
            }
            Insn::CallImmediate(index) => {
                self.inc_pc();
                let Constant::Proc(proc) = &self.constants[index as usize] else {
                    panic!();
                };
                self.call_immediate(proc.as_ref());
                return;
            }
            Insn::Jump(offset) => {
                self.jump(offset);
                return;
            }
            Insn::AddI64 => {
                let (top, bottom) = self.pop2_i64();
                self.push_i64(bottom + top);
            }
            Insn::SubI64 => {
                let (top, bottom) = self.pop2_i64();
                self.push_i64(bottom - top);
            }
            Insn::MulI64 => {
                let (top, bottom) = self.pop2_i64();
                self.push_i64(bottom * top);
            }
            Insn::DivI64 => {
                let (top, bottom) = self.pop2_i64();
                self.push_i64(bottom / top);
            }
            Insn::RemI64 => {
                let (top, bottom) = self.pop2_i64();
                self.push_i64(bottom % top);
            }
            Insn::AddF64 => {
                let (top, bottom) = self.pop2_f64();
                self.push_f64(bottom + top);
            }
            Insn::SubF64 => {
                let (top, bottom) = self.pop2_f64();
                self.push_f64(bottom - top);
            }
            Insn::MulF64 => {
                let (top, bottom) = self.pop2_f64();
                self.push_f64(bottom * top);
            }
            Insn::DivF64 => {
                let (top, bottom) = self.pop2_f64();
                self.push_f64(bottom / top);
            }
            Insn::RemF64 => {
                let (top, bottom) = self.pop2_f64();
                self.push_f64(bottom % top);
            }
            Insn::PrintI64 => {
                println!("{}", self.pop_i64());
            }
            Insn::PrintF64 => {
                println!("{}", self.pop_f64());
            }
            Insn::PrintProc => {
                println!("<proc:{:?}>", self.pop_proc());
            }
        }
        self.inc_pc();
    }

    pub fn run(&mut self) {
        while let Some(frame) = self.frames.last_mut() {
            // Return from procs
            if frame.offset >= unsafe { (*frame.code).len() } {
                self.frames.pop().unwrap();
                continue;
            }
            let insn = unsafe { (*frame.code)[frame.offset] };
            self.evaluate(insn);
        }
    }
}

#[derive(Clone, Copy)]
pub union Value {
    pub as_unit: (),
    pub as_i64: i64,
    pub as_f64: f64,
    pub as_proc: *const Proc,
}

pub enum Constant {
    I64(i64),
    F64(f64),
    Proc(Box<Proc>),
}

impl From<i64> for Constant {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<f64> for Constant {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<Proc> for Constant {
    fn from(value: Proc) -> Self {
        Self::Proc(Box::new(value))
    }
}

pub struct Proc {
    pub code: Vec<Insn>,
}

pub struct StackFrame {
    pub code: *const [Insn],
    pub offset: usize,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Insn {
    // Memory
    StackAlloc(u32),
    Copy,
    Swap,
    Load(u32),
    Store(u32),
    LoadConst(u32),
    // Jumps
    Call,
    CallImmediate(u32),
    Jump(i32),
    // Arithmetic
    AddI64,
    SubI64,
    MulI64,
    DivI64,
    RemI64,
    AddF64,
    SubF64,
    MulF64,
    DivF64,
    RemF64,
    // Debug
    PrintI64,
    PrintF64,
    PrintProc,
}

#[rustfmt::skip]
#[allow(unused_imports)]
pub(super) mod _proc {
    pub use super::Insn::StackAlloc as stack_alloc;
    pub use super::Insn::Copy as copy;
    pub use super::Insn::Swap as swap;
    pub use super::Insn::Load as load;
    pub use super::Insn::Store as store;
    pub use super::Insn::LoadConst as load_const;
    pub use super::Insn::Call as call;
    pub use super::Insn::CallImmediate as call_imm;
    pub use super::Insn::Jump as jump;
    pub use super::Insn::AddI64 as addi;
    pub use super::Insn::SubI64 as subi;
    pub use super::Insn::MulI64 as muli;
    pub use super::Insn::DivI64 as divi;
    pub use super::Insn::RemI64 as remi;
    pub use super::Insn::AddF64 as addf;
    pub use super::Insn::SubF64 as subf;
    pub use super::Insn::MulF64 as mulf;
    pub use super::Insn::DivF64 as divf;
    pub use super::Insn::RemF64 as remf;
    pub use super::Insn::PrintI64 as print_i64;
    pub use super::Insn::PrintF64 as print_f64;
    pub use super::Insn::PrintProc as print_proc;
}
