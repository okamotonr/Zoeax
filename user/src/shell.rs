use common::syscall::put_char;

#[no_mangle]
pub fn main() {
    let msg = "hello world\n";
    for c in msg.bytes() {
        put_char(c as char);
    }
    panic!();
}
