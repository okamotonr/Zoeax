use crate::list::LinkedList;

use super::tcb::{resume, ThreadControlBlock, ThreadInfo};

// TODO: More efficiency
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EndpointState {
    Send,
    Recv,
    Idel,
}

pub struct Endpoint {
    ep_state: EndpointState,
    queue: LinkedList<ThreadInfo>,
}

impl Endpoint {
    pub fn new() -> Self {
        Endpoint {
            ep_state: EndpointState::Idel,
            queue: LinkedList::new(),
        }
    }

    fn pop_from_queue<'a>(
        &mut self,
        ep_state: EndpointState,
    ) -> Option<&'a mut ThreadControlBlock> {
        if self.is_idle() {
            self.ep_state = ep_state
        }

        if self.ep_state == ep_state {
            None
        } else {
            let ret = { self.queue.pop() };
            if self.queue.is_empty() {
                self.ep_state = EndpointState::Idel;
            }
            ret
        }
    }

    pub fn send(&mut self, thread: &mut ThreadControlBlock) -> bool {
        if let Some(reciever_thread) = self.pop_from_queue(EndpointState::Send) {
            reciever_thread.set_ipc_msg(thread.ipc_buffer_ref());
            wake_up_thread(reciever_thread);
            false
        } else {
            block_thread(thread);
            self.queue.push(thread);
            true
        }
    }

    pub fn recv(&mut self, thread: &mut ThreadControlBlock) -> bool {
        if let Some(send_thread) = self.pop_from_queue(EndpointState::Recv) {
            thread.set_ipc_msg(send_thread.ipc_buffer_ref());
            wake_up_thread(send_thread);
            false
        } else {
            block_thread(thread);
            self.queue.push(thread);
            true
        }
    }

    fn is_idle(&self) -> bool {
        self.ep_state == EndpointState::Idel
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self::new()
    }
}

fn wake_up_thread(tcb: &mut ThreadControlBlock) {
    assert!(tcb.next_is_none());
    resume(tcb);
}

fn block_thread(tcb: &mut ThreadControlBlock) {
    // 1, change thread state block
    assert!(tcb.next_is_none());
    tcb.suspend();
    // 2, remove tcb from runqueue
    // currently tcb which will be blocked was poped out from runqueue because it is running thread.
}
