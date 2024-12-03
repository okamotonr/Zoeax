use common::put_char;
use common::sleep;

#[no_mangle]
pub fn main() {
    loop {
        let msg = "hello\n";
        for c in msg.bytes() {
            put_char(c as char);
        }
        sleep(10);

    }
}
