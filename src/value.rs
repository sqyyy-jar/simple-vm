use crate::runtime::proc::Proc;

#[macro_export]
macro_rules! value {
    (@s64 $value: expr) => {
        $crate::value::Value { s64: $value }
    };
    (@f64 $value: expr) => {
        $crate::value::Value { f64: $value }
    };
    (@proc $value: expr) => {
        $crate::value::Value { proc: $value }
    };
}

#[derive(Clone, Copy)]
pub union Value {
    pub s64: i64,
    pub f64: f64,
    pub proc: *const Proc,
}
