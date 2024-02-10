#[macro_export]
macro_rules! proc {
    ($($insn: expr;)*) => {
        {
            #[allow(unused_imports)]
            use $crate::opcodes::_asm::*;
            let mut code = Vec::new();
            $(
                $crate::opcodes::Instruction::write(&$insn, &mut code);
            )*
            $crate::proc::Proc::new(code)
        }
    };
}

pub struct Proc {
    pub code: Box<[u8]>,
}

impl Proc {
    pub fn new(code: impl Into<Box<[u8]>>) -> Self {
        Self { code: code.into() }
    }
}
