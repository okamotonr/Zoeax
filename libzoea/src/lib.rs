#![no_std]

use core::fmt;

use syscall::put_char;

pub use shared::types::BootInfo;
pub use shared::types::IPCBuffer;
pub use shared::registers::Registers;
pub use shared::types::UntypedInfo;
pub use shared::err_kind::ErrKind;
pub use shared::inv_labels::InvLabel;
pub use shared::syscall_no::SysCallNo;
pub mod syscall;
//pub mod syszoea;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

pub struct SyscallWriter;

impl fmt::Write for SyscallWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.as_bytes() {
            let ch = *ch;
            put_char(ch).unwrap();
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut writer = SyscallWriter;
    use fmt::Write;
    writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
