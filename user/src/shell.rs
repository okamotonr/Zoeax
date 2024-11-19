use common::put_char;

#[no_mangle]
pub fn main() {
    let msg = "hello\n";
    for c in msg.bytes() {
        put_char(c as char);
    }

    loop {}
}
