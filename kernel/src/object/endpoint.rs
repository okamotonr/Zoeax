use common::list::{LinkedList, ListItem};

use super::tcb::{ThreadInfo, ThreadControlBlock};

// TODO: More efficiency
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EndpointState {
    Send,
    Recv,
    Idel,
}

pub struct Endpoint {
    ep_state: EndpointState,
    queue: LinkedList<ThreadInfo>
}

impl Endpoint {
    pub fn new() -> Self {
        Endpoint {
            ep_state: EndpointState::Idel,
            queue: LinkedList::new()
        }
    }

    fn pop_from_queue<'a>(&mut self, ep_state: EndpointState) -> Option<&'a mut ThreadControlBlock> {
        if self.is_idle() {
            self.ep_state = ep_state
        }

        if self.ep_state == ep_state {
            None
        } else {
            let ret = {
                self.queue.pop()
            };
            if self.queue.is_empty() {
                self.ep_state = EndpointState::Idel;
            }
            ret
        }
    }

    pub fn send(&mut self, thread: &mut ThreadControlBlock) {
        if let Some(reciever_thread) = self.pop_from_queue(EndpointState::Send) {
            reciever_thread.set_msg(thread.msg_buffer);
            wake_up_thread(thread);
            wake_up_thread(reciever_thread);
        } else {
            block_thread(thread);
            self.queue.push(thread);
        }
    }

    pub fn recv(&mut self, thread: &mut ThreadControlBlock) {
        if let Some(send_thread) = self.pop_from_queue(EndpointState::Recv) {
            thread.set_msg(send_thread.msg_buffer);
            wake_up_thread(thread);
            wake_up_thread(send_thread);
        } else {
            block_thread(thread);
            self.queue.push(thread);
        }
    }

    fn is_idle(&self) -> bool {
        self.ep_state == EndpointState::Idel
    }
}

fn wake_up_thread<T>(_:&mut ListItem<T>) {
    // 1, change thread state to Runnable
    // 2, put into runqueu
    todo!()
}
fn block_thread<T>(_: &mut ListItem<T>) {
    // 1, change thread state block
    todo!()
}
