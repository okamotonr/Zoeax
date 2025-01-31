use core::num::NonZeroU64;

use crate::list::LinkedList;

use super::tcb::{resume, ThreadControlBlock, ThreadInfo};

pub struct Notification {
    notify_bit: Option<NonZeroU64>,
    wait_queue: LinkedList<ThreadInfo>,
}

impl Notification {
    pub fn new() -> Self {
        Notification {
            notify_bit: None,
            wait_queue: LinkedList::new(),
        }
    }

    fn set_notify(&mut self, notify_bit: u64) {
        self.notify_bit = NonZeroU64::new(notify_bit)
    }

    pub fn send_signal(&mut self, val: u64) {
        // TODO: val must be nonzero
        if let Some(wait_thread) = self.wait_queue.pop() {
            wait_thread.registers.a1 = val as usize;
            wake_up_thread(wait_thread);
        } else {
            let old_v = if let Some(v) = self.notify_bit {
                u64::from(v)
            } else {
                0
            };
            let new_v = old_v | val;
            self.set_notify(new_v)
        }
    }

    pub fn wait_signal(&mut self, thread: &mut ThreadControlBlock) -> bool {
        if let Some(bit) = self.notify_bit.take() {
            thread.registers.a1 = u64::from(bit) as usize;
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
