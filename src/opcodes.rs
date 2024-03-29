//! This module contains all opcodes supported by the runtime.

use crate::util::{Read, Write};

#[rustfmt::skip]
macro_rules! opcodes {
    ($path: path) => {
        $path! {
            // Memory
            (alloc,        alloc,      { size: u16 })
            (r#move,       mov,        { dst: i16, src: i16 })
            (move_value,   movv,       { dst: i16, value: i64 })
            (load_const,   ldc,        { dst: i16, index: u32 })
            (load_proc,    ldp,        { dst: i16, index: u32 })
            // Jumps
            (call,         call,       { index: u32 })
            (call_dynamic, call_dyn,   { src: i16 })
            (branch,       b,          { offset: i32 })
            (branch_z,     bz,         { src: i16, offset: i32 })
            (branch_nz,    bnz,        { src: i16, offset: i32 })
            (branch_lz,    blz,        { src: i16, offset: i32 })
            (branch_lez,   blez,       { src: i16, offset: i32 })
            (branch_gz,    bgz,        { src: i16, offset: i32 })
            (branch_gez,   bgez,       { src: i16, offset: i32 })
            (r#return,     ret,        {})
            // Arithmetic
            // s64
            (add_s64,      adds,       { dst: i16, left: i16, right: i16 })
            (sub_s64,      subs,       { dst: i16, left: i16, right: i16 })
            (mul_s64,      muls,       { dst: i16, left: i16, right: i16 })
            (div_s64,      divs,       { dst: i16, left: i16, right: i16 })
            (rem_s64,      rems,       { dst: i16, left: i16, right: i16 })
            // f64
            (add_f64,      addf,       { dst: i16, left: i16, right: i16 })
            (sub_f64,      subf,       { dst: i16, left: i16, right: i16 })
            (mul_f64,      mulf,       { dst: i16, left: i16, right: i16 })
            (div_f64,      divf,       { dst: i16, left: i16, right: i16 })
            (rem_f64,      remf,       { dst: i16, left: i16, right: i16 })
            // Debug
            (print_s64,    print_s64,  { src: i16 })
            (print_f64,    print_f64,  { src: i16 })
            (print_proc,   print_proc, { src: i16 })
            // Special
            (halt,         hlt,        {})
            (breakpoint,   brkp,       {})
        }
    };
}

macro_rules! create_constants {
    ($( ($name: ident, $asm_name: ident, { $($arg_name: ident: $arg_type: ty),* }) )*) => {
        create_constants!(#paste[0] $(($name, $asm_name, {$($arg_name: $arg_type),*}))*);
    };
    (#paste[$opc: expr] ($name: ident, $asm_name: ident, {$($arg_name: ident: $arg_type: ty),*})
        $(($n_name: ident, $n_asm_name: ident, {$($n_arg_name: ident: $n_arg_type: ty),*}))*) => {
        ::paste::paste! {
            pub const [<$name:upper>]: u8 = $opc;

            pub struct [<$name:camel>] {
                $(
                    pub $arg_name: $arg_type,
                )*
            }

            impl [<$name:camel>] {
                pub fn opcode(&self) -> u8 {
                    [<$name:upper>]
                }
            }

            impl Instruction for [<$name:camel>] {
                fn write<T: $crate::util::Write>(&self, out: &mut T) {
                    out.write_u8(self.opcode());
                    $(
                        out.[<write_$arg_type>](self.$arg_name);
                    )*
                }

                #[inline]
                #[allow(unused_variables)]
                fn read<T: $crate::util::Read>(src: &mut T) -> Self {
                    $(
                        let $arg_name: $arg_type = $crate::util::Read::read::<$arg_type>(src);
                    )*
                    Self {
                        $($arg_name),*
                    }
                }
            }
        }

        create_constants!(#paste[$opc + 1] $(($n_name, $n_asm_name, {$($n_arg_name: $n_arg_type),*}))*);
    };
    (#paste[$opc:expr]) => {};
}

macro_rules! create_asm {
    ($( ($name: ident, $asm_name: ident, { $($arg_name: ident: $arg_type: ty),* }) )*) => {
        pub mod _asm {
            pub fn todo<A, T>(_: A) -> T { todo!(); }
            create_asm!(#paste $(($name, $asm_name, {$($arg_name: $arg_type),*}))*);
        }
    };
    (#paste ($name: ident, $asm_name: ident, {$($arg_name: ident: $arg_type: ty),*})
        $(($n_name: ident, $n_asm_name: ident, {$($n_arg_name: ident: $n_arg_type: ty),*}))*) => {
        ::paste::paste! {
            pub fn $asm_name(
                $($arg_name: $arg_type),*
            ) -> super::[<$name:camel>] {
                super::[<$name:camel>] {
                    $($arg_name),*
                }
            }
        }
        create_asm!(#paste $(($n_name, $n_asm_name, {$($n_arg_name: $n_arg_type),*}))*);
    };
    (#paste) => {};
}

pub trait Instruction {
    fn write<T: Write>(&self, out: &mut T);

    fn read<T: Read>(src: &mut T) -> Self;
}

opcodes!(create_constants);
opcodes!(create_asm);
