use common::syscall::put_char;
use common::syscall::sleep;
use common::syscall::send;
use common::syscall::Message;
use common::syscall::recieve;

#[no_mangle]
pub fn main() {
    let msg = "I am ping server\n";
    for c in msg.bytes() {
        put_char(c as char);

    }

    sleep(100);
    let msg = "ping: wake up\n";
    for c in msg.bytes() {
        put_char(c as char);
    }

    let mut message = Message::new();
    let msg = "ping\n";
    let mut count = 0;
    for c in msg.bytes() {
        message.data[count] = c;
        count += 2;
    }
    send(2, &message);
    let mut recv = Message::new();
    recieve(&mut recv);

    let msg = "ping: get message\n";
    for c in msg.bytes() {
        put_char(c as char);
    }

    for c in "ping: ".bytes() {
        put_char(c as char);
    }
    for c in recv.data {
        put_char(c as char);
    }
    loop {}
}

