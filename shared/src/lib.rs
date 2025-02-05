#![no_std]

pub mod cap_type;
pub mod err_kind;
pub mod inv_labels;
pub mod registers;
pub mod syscall_no;
pub mod types;

#[macro_export]
macro_rules! const_assert_single {
    ($cond:expr, $msg:expr $(,)?) => {
        const _: () = {
            if !$cond {
                panic!($msg);
            }
        };
    };
    ($cond:expr $(,)?) => {
        const _: () = {
            if !$cond {
                panic!(concat!(
                    "Compile-time assertion failed: ",
                    stringify!($cond)
                ));
            }
        };
    };
}

#[macro_export]
macro_rules! const_assert {
    ($($cond:expr),+ $(,)?) => {
        $( $crate::const_assert_single!($cond); )+
    };
    ( $( $cond:expr => $msg:expr ),+ $(,)? ) => {
        $( $crate::const_assert_single!($cond, $msg); )+
    };
}
