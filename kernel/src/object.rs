mod cnode;
mod endpoint;
mod notification;
pub mod page_table;
mod tcb;

pub use crate::object::cnode::{CNode, CNodeEntry, ManagementDB};
pub use crate::object::endpoint::Endpoint;
pub use crate::object::notification::Notification;
pub use crate::object::page_table::PageTable;
pub use crate::object::tcb::{resume, Registers, ThreadControlBlock, ThreadInfo};

pub struct Untyped;

// marker trait
pub trait KObject {}

// pub trait KObject {
//     fn get_size(_user_size: usize) -> usize;
// }

//
// #![feature(specilization)]
// default impl<T: Sized> KObject for T {
//     fn get_size(_user_size: usize) -> usize {
//         mem::size_of::<T>()
//     }
// }
