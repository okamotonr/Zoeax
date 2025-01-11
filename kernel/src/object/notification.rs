use core::num::NonZeroU64;

use common::list::{LinkedList, ListItem};

use super::tcb::{ThreadControlBlock, ThreadInfo};

pub struct Notification {
    notify_bit: Option<NonZeroU64>,
    wait_queue: LinkedList<ThreadInfo>
}

impl Notification {
    pub fn new() -> Self {
        Notification {
            notify_bit: None,
            wait_queue: LinkedList::new()
        }
    }

    fn set_notify(&mut self, notify_bit: u64) {
        self.notify_bit = NonZeroU64::new(notify_bit)
    }

    pub fn send_notify(&mut self, val: u64) {
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

    pub fn wait_notify(&mut self, thread: &mut ThreadControlBlock) {
        if let Some(bit) = self.notify_bit.take() {
            thread.registers.a1 = u64::from(bit) as usize;
            wake_up_thread(thread);
        } else {
            block_thread(thread);
            self.wait_queue.push(thread)
        }
    }
}

impl Default for Notification {
    fn default() -> Self {
        Self::new()
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
