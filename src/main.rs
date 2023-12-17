#![feature(new_uninit)]

pub mod opcodes;
pub mod proc;
pub mod rt;
pub mod stack;
pub mod util;
pub mod value;

fn main() {
    let mut rt = make_runtime! {
        let constants = [
            21i64,
            2i64
        ];
        let procs = [
            main = {
                alloc(2);
                ldc(0, 0);
                ldc(1, 1);
                call(3);
                print_i64(1);
                hlt();
            },
            mul = {
                muli(-1, -1, -2);
                ret();
            }
        ];
    };
    rt.call(2);
    rt.run();
}
