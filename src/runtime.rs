pub mod debug;
pub mod proc;
pub mod stack;

use std::{mem::size_of, ptr::null};

use crate::{
    opcodes::{
        AddF64, AddS64, Alloc, Branch, BranchGez, BranchGz, BranchLez, BranchLz, BranchNz, BranchZ,
        Call, CallDynamic, DivF64, DivS64, Instruction, LoadConst, LoadProc, Move, MoveValue,
        MulF64, MulS64, PrintF64, PrintProc, PrintS64, RemF64, RemS64, SubF64, SubS64, ADD_F64,
        ADD_S64, ALLOC, BRANCH, BRANCH_GEZ, BRANCH_GZ, BRANCH_LEZ, BRANCH_LZ, BRANCH_NZ, BRANCH_Z,
        BREAKPOINT, CALL, CALL_DYNAMIC, DIV_F64, DIV_S64, HALT, LOAD_CONST, LOAD_PROC, MOVE,
        MOVE_VALUE, MUL_F64, MUL_S64, PRINT_F64, PRINT_PROC, PRINT_S64, REM_F64, REM_S64, RETURN,
        SUB_F64, SUB_S64,
    },
    util::Read,
    value,
};

use self::{proc::Proc, stack::Stack};

#[macro_export]
macro_rules! make_runtime {
    (
        .constants = [ $($constant: expr),* ];
        .procs = [ $( .{ $($insn: expr;)* } ),* ];
    ) => {{
        let mut runtime = $crate::runtime::Runtime::new();
        $(
            runtime.push_constant(std::convert::Into::into($constant));
        )*
        $({
            #[allow(unused_imports)]
            use $crate::opcodes::_asm::*;
            let mut code = Vec::new();
            $(
                $crate::opcodes::Instruction::write(&$insn, &mut code);
            )*
            runtime.push_proc(::std::convert::Into::into($crate::runtime::proc::Proc::new(code)));
        })*
        runtime
    }};
}

#[repr(C)]
pub struct Runtime {
    /// Program counter
    pc: *const u8,
    stack: Stack,
    constants: Vec<Constant>,
    procs: Vec<Box<Proc>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            pc: null(),
            stack: Stack::new(4096),
            constants: Vec::new(),
            procs: Vec::new(),
        }
    }

    pub fn push_constant(&mut self, constant: Constant) {
        self.constants.push(constant);
    }

    pub fn push_proc(&mut self, proc: Proc) {
        self.procs.push(Box::new(proc));
    }

    pub fn call(&mut self, index: u32) {
        let Some(proc) = self.procs.get(index as usize) else {
            panic!("Unable to call constants #{index}");
        };
        self.push_call_frame(&**proc);
    }

    pub fn load_const(&mut self, dst: i16, index: u32) {
        let Some(constant) = self.constants.get(index as usize) else {
            panic!("Unable to load constant #{index}");
        };
        self.stack.store(
            dst,
            match constant {
                Constant::S64(value) => value!(@s64 *value),
                Constant::F64(value) => value!(@f64 *value),
            },
        );
    }

    pub fn load_proc(&mut self, dst: i16, index: u32) {
        let Some(proc) = self.procs.get(index as usize) else {
            panic!("Unable to load proc #{index}");
        };
        self.stack.store(dst, value!(@proc &**proc));
    }

    /// Pushes a call frame and sets the instruction pointer
    pub fn push_call_frame(&mut self, proc: *const Proc) {
        unsafe {
            self.stack.push_frame(self.pc);
            self.pc = (*proc).code.as_ptr();
        }
    }

    #[inline]
    pub fn branch_rel(&mut self, offset: i32) {
        unsafe {
            self.pc = self.pc.offset(offset as isize);
        }
    }

    pub fn fetch(&mut self) -> u8 {
        self.read::<u8>()
    }

    pub fn execute(&mut self, opcode: u8) {
        unsafe {
            match opcode {
                ALLOC => {
                    let insn = Alloc::read(self);
                    self.stack.alloc(insn.size as usize);
                }
                MOVE => {
                    let insn = Move::read(self);
                    let value = self.stack.load(insn.src);
                    self.stack.store(insn.dst, value);
                }
                MOVE_VALUE => {
                    let insn = MoveValue::read(self);
                    self.stack.store(insn.dst, value!(@s64 insn.value));
                }
                LOAD_CONST => {
                    let insn = LoadConst::read(self);
                    self.load_const(insn.dst, insn.index);
                }
                LOAD_PROC => {
                    let insn = LoadProc::read(self);
                    self.load_proc(insn.dst, insn.index);
                }
                CALL => {
                    let insn = Call::read(self);
                    self.call(insn.index);
                }
                CALL_DYNAMIC => {
                    let insn = CallDynamic::read(self);
                    let proc = self.stack.load(insn.src).proc;
                    self.push_call_frame(proc);
                }
                BRANCH => {
                    let insn = Branch::read(self);
                    self.branch_rel(insn.offset);
                }
                BRANCH_Z => {
                    let insn = BranchZ::read(self);
                    if self.stack.load(insn.src).s64 == 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_NZ => {
                    let insn = BranchNz::read(self);
                    if self.stack.load(insn.src).s64 != 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_LZ => {
                    let insn = BranchLz::read(self);
                    if self.stack.load(insn.src).s64 < 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_LEZ => {
                    let insn = BranchLez::read(self);
                    if self.stack.load(insn.src).s64 <= 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_GZ => {
                    let insn = BranchGz::read(self);
                    if self.stack.load(insn.src).s64 > 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_GEZ => {
                    let insn = BranchGez::read(self);
                    if self.stack.load(insn.src).s64 >= 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                RETURN => {
                    let ra = self.stack.return_call();
                    self.pc = ra;
                }
                ADD_S64 => {
                    let insn = AddS64::read(self);
                    let left = self.stack.load(insn.left).s64;
                    let right = self.stack.load(insn.right).s64;
                    self.stack.store(insn.dst, value!(@s64 left + right));
                }
                SUB_S64 => {
                    let insn = SubS64::read(self);
                    let left = self.stack.load(insn.left).s64;
                    let right = self.stack.load(insn.right).s64;
                    self.stack.store(insn.dst, value!(@s64 left - right));
                }
                MUL_S64 => {
                    let insn = MulS64::read(self);
                    let left = self.stack.load(insn.left).s64;
                    let right = self.stack.load(insn.right).s64;
                    self.stack.store(insn.dst, value!(@s64 left * right));
                }
                DIV_S64 => {
                    let insn = DivS64::read(self);
                    let left = self.stack.load(insn.left).s64;
                    let right = self.stack.load(insn.right).s64;
                    self.stack.store(insn.dst, value!(@s64 left / right));
                }
                REM_S64 => {
                    let insn = RemS64::read(self);
                    let left = self.stack.load(insn.left).s64;
                    let right = self.stack.load(insn.right).s64;
                    self.stack.store(insn.dst, value!(@s64 left % right));
                }
                ADD_F64 => {
                    let insn = AddF64::read(self);
                    let left = self.stack.load(insn.left).f64;
                    let right = self.stack.load(insn.right).f64;
                    self.stack.store(insn.dst, value!(@f64 left + right));
                }
                SUB_F64 => {
                    let insn = SubF64::read(self);
                    let left = self.stack.load(insn.left).f64;
                    let right = self.stack.load(insn.right).f64;
                    self.stack.store(insn.dst, value!(@f64 left - right));
                }
                MUL_F64 => {
                    let insn = MulF64::read(self);
                    let left = self.stack.load(insn.left).f64;
                    let right = self.stack.load(insn.right).f64;
                    self.stack.store(insn.dst, value!(@f64 left * right));
                }
                DIV_F64 => {
                    let insn = DivF64::read(self);
                    let left = self.stack.load(insn.left).f64;
                    let right = self.stack.load(insn.right).f64;
                    self.stack.store(insn.dst, value!(@f64 left / right));
                }
                REM_F64 => {
                    let insn = RemF64::read(self);
                    let left = self.stack.load(insn.left).f64;
                    let right = self.stack.load(insn.right).f64;
                    self.stack.store(insn.dst, value!(@f64 left % right));
                }
                PRINT_S64 => {
                    let insn = PrintS64::read(self);
                    let value = self.stack.load(insn.src).s64;
                    println!("{value}");
                }
                PRINT_F64 => {
                    let insn = PrintF64::read(self);
                    let value = self.stack.load(insn.src).f64;
                    println!("{value}");
                }
                PRINT_PROC => {
                    let insn = PrintProc::read(self);
                    let value = self.stack.load(insn.src).proc;
                    println!("<proc:{value:?}>");
                }
                HALT => {
                    self.pc = null();
                }
                BREAKPOINT => (),
                _ => unimplemented!("opcode [0x{opcode:02x}]"),
            }
        }
    }

    pub fn run(&mut self) {
        while !self.pc.is_null() {
            let opcode = self.fetch();
            self.execute(opcode);
        }
    }

    pub fn run_debug(&mut self) {
        while !self.pc.is_null() {
            let opcode = self.fetch();
            self.execute(opcode);
        }
    }
}

impl Read for Runtime {
    #[inline]
    fn read<T: Copy>(&mut self) -> T {
        unsafe {
            let value = (self.pc as *const T).read_unaligned();
            self.pc = self.pc.add(size_of::<T>());
            value
        }
    }
}

pub enum Constant {
    S64(i64),
    F64(f64),
}

impl From<i64> for Constant {
    fn from(value: i64) -> Self {
        Self::S64(value)
    }
}

impl From<f64> for Constant {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}
