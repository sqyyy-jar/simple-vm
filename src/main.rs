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
                alloc(1);
                movv(0, 5);
                call(7);
                print_i64(0);
                hlt();
            },
            .{ // [6]: mul()
                muli(-1, -1, -2);
                ret();
            },
            .{ // [7]: factorial(n)
                alloc(2);
                bnz(-1, 1 + 2 + 8 + 1);                 // if (n == 0)
                movv(-1, 1);                            // return 1
                ret();
                movv(0, 1);                             // %0 = 1
                mov(1, -1);                             // %1 = n
                subi(1, 1, 0);                          // %1 = %1 - %0
                call(7);                                // %1 = factorial(%1)
                muli(-1, -1, 1);                        // return n * %1
                ret();
            }
        ];
    };
    rt.call(5);
    rt.run();
}
