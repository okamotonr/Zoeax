use common::syscall::put_char;
use common::syscall::sleep;
use common::syscall::send;
use common::syscall::recieve;
use common::syscall::Message;

#[no_mangle]
pub fn main() {
    let msg = "I am pong server\n";
    for c in msg.bytes() {
        put_char(c as char);
    }

    let mut message = Message::new();
    recieve(&mut message);
    let msg = "pong: get message\n";
    for c in msg.bytes() {
         put_char(c as char);
    }
    for c in "pong: ".bytes() {
         put_char(c as char);
    }
    // Error occured if not comment out
    // for c in message.data {
    //      put_char(c as char);
    // }

    sleep(100);
    let msg = "pong: wake up\n";
    for c in msg.bytes() {
        put_char(c as char)
    }

    let mut message = Message::new();
    let msg = "pong\n";
    for (i, c) in msg.bytes().enumerate() {
        message.data[i] = c;
    }
    send(1, &message);

    loop {}
}

