//pub mod asm;
//pub mod parser;
pub mod rt;

fn main() {
    let mut rt = make_runtime! {
        let constants = [
            21i64,
            2i64
        ];
        let procs = [
            main = {
                load_const(0);
                load_const(1);
                call_imm(3);
                print_i64;
            },
            mul = {
                muli;
            }
        ];
    };
    rt.push_call_frame(2);
    rt.run();
}
