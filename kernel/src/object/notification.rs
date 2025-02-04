use crate::list::LinkedList;

use super::tcb::{resume, ThreadControlBlock, ThreadInfo};

pub struct Notification {
    notify_bit: u64,
    wait_queue: LinkedList<ThreadInfo>,
    bit_is_set: bool,
}

impl Notification {
    pub fn new() -> Self {
        Notification {
            notify_bit: 0,
            wait_queue: LinkedList::new(),
            bit_is_set: false,
        }
    }

    fn set_notify(&mut self, notify_bit: u64) {
        self.notify_bit = notify_bit
    }

    pub fn send_signal(&mut self, val: u64) {
        if let Some(wait_thread) = self.wait_queue.pop() {
            wait_thread.registers.a1 = val as usize;
            wake_up_thread(wait_thread);
        } else {
            let old_v = self.notify_bit;
            let new_v = old_v | val;
            self.bit_is_set = true;
            self.set_notify(new_v)
        }
    }

    pub fn wait_signal(&mut self, thread: &mut ThreadControlBlock) -> bool {
        if self.bit_is_set {
            thread.registers.a1 = self.notify_bit as usize;
            self.notify_bit = 0;
            self.bit_is_set = false;
            false
        } else {
            block_thread(thread);
            self.wait_queue.push(thread);
            true
        }
    }
}

impl Default for Notification {
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
