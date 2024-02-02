use std::{mem::size_of, ptr::null};

use crate::{
    opcodes::{
        AddF64, AddI64, Alloc, Branch, BranchGez, BranchGz, BranchLez, BranchLz, BranchNz, BranchZ,
        Call, CallDynamic, DivF64, DivI64, Instruction, LoadConst, Move, MoveValue, MulF64, MulI64,
        PrintF64, PrintI64, PrintProc, RemF64, RemI64, SubF64, SubI64, ADD_F64, ADD_I64, ALLOC,
        BRANCH, BRANCH_GEZ, BRANCH_GZ, BRANCH_LEZ, BRANCH_LZ, BRANCH_NZ, BRANCH_Z, CALL,
        CALL_DYNAMIC, DIV_F64, DIV_I64, HALT, LOAD_CONST, MOVE, MOVE_VALUE, MUL_F64, MUL_I64,
        PRINT_F64, PRINT_I64, PRINT_PROC, REM_F64, REM_I64, RETURN, SUB_F64, SUB_I64,
    },
    proc::Proc,
    stack::Stack,
    util::Read,
    value,
};

#[macro_export]
macro_rules! make_runtime {
    (
        let constants = [ $($constant: expr),* ];
        let procs = [ $( .{ $($insn: expr;)* } ),* ];
    ) => {{
        let mut runtime = $crate::rt::Runtime::new();
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
            runtime.push_constant(::std::convert::Into::into($crate::proc::Proc::new(code)));
        })*
        runtime
    }};
}

#[repr(C)]
pub struct Runtime {
    /// Instruction pointer
    pub ip: *const u8,
    pub stack: Stack,
    pub constants: Vec<Constant>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            ip: null(),
            constants: Vec::new(),
            stack: Stack::new(4096),
        }
    }

    pub fn push_constant(&mut self, constant: Constant) {
        self.constants.push(constant);
    }

    pub fn call(&mut self, index: u32) {
        let Constant::Proc(proc) = &self.constants[index as usize] else {
            panic!("Unable to call constants[{index}]");
        };
        self.push_call_frame(&**proc);
    }

    pub fn load_const(&mut self, dst: i16, index: u32) {
        let constant = &self.constants[index as usize];
        self.stack.store(
            dst,
            match constant {
                Constant::I64(value) => value!(@i64 *value),
                Constant::F64(value) => value!(@f64 *value),
                Constant::Proc(value) => value!(@proc value.as_ref()),
            },
        );
    }

    /// Pushes a call frame and sets the instruction pointer
    pub fn push_call_frame(&mut self, proc: *const Proc) {
        unsafe {
            self.stack.push_frame(self.ip);
            self.ip = (*proc).code.as_ptr();
        }
    }

    #[inline]
    pub fn branch_rel(&mut self, offset: i32) {
        unsafe {
            self.ip = self.ip.offset(offset as isize);
        }
    }

    pub fn evaluate(&mut self) {
        unsafe {
            let opc = self.read::<u8>();
            match opc {
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
                    self.stack.store(insn.dst, value!(@i64 insn.value));
                }
                LOAD_CONST => {
                    let insn = LoadConst::read(self);
                    self.load_const(insn.dst, insn.index);
                }
                CALL => {
                    let insn = Call::read(self);
                    self.call(insn.index);
                }
                CALL_DYNAMIC => {
                    let insn = CallDynamic::read(self);
                    let proc = self.stack.load(insn.src).as_proc;
                    self.push_call_frame(proc);
                }
                BRANCH => {
                    let insn = Branch::read(self);
                    self.branch_rel(insn.offset);
                }
                BRANCH_Z => {
                    let insn = BranchZ::read(self);
                    if self.stack.load(insn.src).as_i64 == 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_NZ => {
                    let insn = BranchNz::read(self);
                    if self.stack.load(insn.src).as_i64 != 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_LZ => {
                    let insn = BranchLz::read(self);
                    if self.stack.load(insn.src).as_i64 < 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_LEZ => {
                    let insn = BranchLez::read(self);
                    if self.stack.load(insn.src).as_i64 <= 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_GZ => {
                    let insn = BranchGz::read(self);
                    if self.stack.load(insn.src).as_i64 > 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                BRANCH_GEZ => {
                    let insn = BranchGez::read(self);
                    if self.stack.load(insn.src).as_i64 >= 0 {
                        self.branch_rel(insn.offset);
                    }
                }
                RETURN => {
                    let ra = self.stack.return_call();
                    self.ip = ra;
                }
                ADD_I64 => {
                    let insn = AddI64::read(self);
                    let left = self.stack.load(insn.left).as_i64;
                    let right = self.stack.load(insn.right).as_i64;
                    self.stack.store(insn.dst, value!(@i64 left + right));
                }
                SUB_I64 => {
                    let insn = SubI64::read(self);
                    let left = self.stack.load(insn.left).as_i64;
                    let right = self.stack.load(insn.right).as_i64;
                    self.stack.store(insn.dst, value!(@i64 left - right));
                }
                MUL_I64 => {
                    let insn = MulI64::read(self);
                    let left = self.stack.load(insn.left).as_i64;
                    let right = self.stack.load(insn.right).as_i64;
                    self.stack.store(insn.dst, value!(@i64 left * right));
                }
                DIV_I64 => {
                    let insn = DivI64::read(self);
                    let left = self.stack.load(insn.left).as_i64;
                    let right = self.stack.load(insn.right).as_i64;
                    self.stack.store(insn.dst, value!(@i64 left / right));
                }
                REM_I64 => {
                    let insn = RemI64::read(self);
                    let left = self.stack.load(insn.left).as_i64;
                    let right = self.stack.load(insn.right).as_i64;
                    self.stack.store(insn.dst, value!(@i64 left % right));
                }
                ADD_F64 => {
                    let insn = AddF64::read(self);
                    let left = self.stack.load(insn.left).as_f64;
                    let right = self.stack.load(insn.right).as_f64;
                    self.stack.store(insn.dst, value!(@f64 left + right));
                }
                SUB_F64 => {
                    let insn = SubF64::read(self);
                    let left = self.stack.load(insn.left).as_f64;
                    let right = self.stack.load(insn.right).as_f64;
                    self.stack.store(insn.dst, value!(@f64 left - right));
                }
                MUL_F64 => {
                    let insn = MulF64::read(self);
                    let left = self.stack.load(insn.left).as_f64;
                    let right = self.stack.load(insn.right).as_f64;
                    self.stack.store(insn.dst, value!(@f64 left * right));
                }
                DIV_F64 => {
                    let insn = DivF64::read(self);
                    let left = self.stack.load(insn.left).as_f64;
                    let right = self.stack.load(insn.right).as_f64;
                    self.stack.store(insn.dst, value!(@f64 left / right));
                }
                REM_F64 => {
                    let insn = RemF64::read(self);
                    let left = self.stack.load(insn.left).as_f64;
                    let right = self.stack.load(insn.right).as_f64;
                    self.stack.store(insn.dst, value!(@f64 left % right));
                }
                PRINT_I64 => {
                    let insn = PrintI64::read(self);
                    let value = self.stack.load(insn.src).as_i64;
                    println!("{value}");
                }
                PRINT_F64 => {
                    let insn = PrintF64::read(self);
                    let value = self.stack.load(insn.src).as_f64;
                    println!("{value}");
                }
                PRINT_PROC => {
                    let insn = PrintProc::read(self);
                    let value = self.stack.load(insn.src).as_proc;
                    println!("<proc:{value:?}>");
                }
                HALT => {
                    self.ip = null();
                }
                _ => unimplemented!("opcode::0x{opc:02x}"),
            }
        }
    }

    pub fn run(&mut self) {
        while !self.ip.is_null() {
            self.evaluate();
        }
    }
}

impl Read for Runtime {
    #[inline]
    fn read<T: Copy>(&mut self) -> T {
        unsafe {
            let value = (self.ip as *const T).read_unaligned();
            self.ip = self.ip.add(size_of::<T>());
            value
        }
    }
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
