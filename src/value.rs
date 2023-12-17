use crate::proc::Proc;

#[macro_export]
macro_rules! value {
    (@i64 $value: expr) => {
        $crate::value::Value { as_i64: $value }
    };
    (@f64 $value: expr) => {
        $crate::value::Value { as_f64: $value }
    };
    (@proc $value: expr) => {
        $crate::value::Value { as_proc: $value }
    };
}

#[derive(Clone, Copy)]
pub union Value {
    pub as_i64: i64,
    pub as_f64: f64,
    pub as_proc: *const Proc,
}
