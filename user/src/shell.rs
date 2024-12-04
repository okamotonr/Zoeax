use common::syscall::put_char;
use common::syscall::sleep;
use core::arch::asm;

#[no_mangle]
pub fn main() {
    loop {
        let msg = "hello\n";
        for c in msg.bytes() {
            put_char(c as char);
        }
        
        sleep(10000);

        let msg = "wake up\n";
        for c in msg.bytes() {
            put_char(c as char);
        }
    }
}
