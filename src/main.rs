#![feature(new_uninit)]

pub mod opcodes;
pub mod proc;
pub mod rt;
pub mod stack;
pub mod util;
pub mod value;

fn main() {
    let mut rt = make_runtime! {
        .constants = [];
        .procs = [
            .{ // [0]: main()
                alloc(1);
                movv(0, 3);
                call(2);
                print_s64(0);
                hlt();
            },
            .{ // [1]: factorial(n)
                alloc(2);                               // {one, a}
                bnz(-1, 1 + 2 + 8 + 1);                 // if (n == 0)
                movv(-1, 1);                            // return 1
                ret();
                movv(0, 1);                             // one = 1
                mov(1, -1);                             // a = n
                subs(1, 1, 0);                          // a -= one
                call(1);                                // a = factorial(a)
                muls(-1, -1, 1);                        // return n * a
                ret();
            },
            .{ // [2]: fibonacci(n)
                alloc(4);                               // {one, a, b, c}
                movv(0, 1);                             // one = 1
                subs(1, -1, 0);                         // a = n <> 1
                bgz(1, 1);                              // if (n <= one)
                ret();                                  // return n
                subs(1, -1, 0);                         // a = n - one
                mov(3, 1);                              // c = a
                call(2);                                // c = fibonacci(c)
                print_s64(3);
                mov(2, 3);                              // b = c
                subs(3, 1, 0);                          // c = a - one
                call(2);                                // c = fibonacci(c)
                print_s64(3);
                adds(-1, 2, 3);                         // return b + c
                print_s64(-1);
                ret();
            }
        ];
    };
    rt.call(0);
    rt.run();
}
