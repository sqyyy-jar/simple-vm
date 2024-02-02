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
            0i64,
            0i64,
            0i64,
            0i64,
            0i64
        ];
        let procs = [
            .{ // [5]: main()
                alloc(2);
                ldc(0, 0);
                ldc(1, 1);
                call(3);
                print_i64(1);
                hlt();
            },
            .{ // [6]: mul()
                muli(-1, -1, -2);
                ret();
            },
            .{ // [7]: factorial(n)
                alloc(2);
                bz(-1, todo!());
                ret();
            }
        ];
    };
    rt.call(2);
    rt.run();
}
