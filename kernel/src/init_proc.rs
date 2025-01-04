use common::elf::*;

use crate::capability::tcb::TCBCap;
use crate::capability::untyped::UntypedCap;
use crate::capability::Capability;
use crate::capability::cnode::CNodeCap;
use crate::capability::RawCapability;
use crate::common::align_up;
use crate::memory::VirtAddr;
use crate::memory::PAGE_SIZE;
use crate::object::CNode;
use crate::object::CNodeEntry;
use crate::process::Process;
use crate::vm::KernelVAddress;
use crate::vm::{alloc_vm, PAGE_R, PAGE_U, PAGE_W, PAGE_X};
use core::array::IntoIter;
use core::cmp::min;
use core::ptr;
use core::mem::{MaybeUninit};

// TODO
const ROOT_CNODE_ENTRY_NUM: usize = 64;
const ROOT_CNODE_IDX: usize = 3;

// Only initalization
impl CNode {
    pub fn write_slot(&mut self, cap: RawCapability, index: usize) {
        let root = (self as *mut Self).cast::<CNodeEntry>();
        let entry = CNodeEntry::new_with_rawcap(cap);
        unsafe {
            *root.add(index) = entry
        }
    } 
}

struct RootServerMemory<'a> {
    cnode: &'a mut MaybeUninit<CNode>,
    vspace: &'a mut MaybeUninit<PageTable>
}

impl<'a> RootServerMemory<'a> {
    pub fn alloc_obj(start_address: KernelVAddress) -> (KernelVAddress, &'a mut MaybeUninit<CNode>) {
        let cnode_ptr = <KernelVAddress as Into<*mut CNode>>::into(start_address);
        let next_address = start_address.add(CNodeCap::get_object_size(ROOT_CNODE_ENTRY_NUM));
        unsafe {
            (next_address, cnode_ptr.as_uninit_mut().unwrap())
        }
    }

    pub fn init_with_uninit(start_address: KernelVAddress) -> (KernelVAddress, Self) {
        let (next_address, cnode) = Self::alloc_obj(start_address);
        (next_address, Self {cnode})
    }

    pub fn get_cnode_cap(&mut self) -> CNodeCap {
        let cnode = self.cnode.write(CNode::new());
        let cap = CNodeCap::init((cnode as *mut CNode).into(), ROOT_CNODE_ENTRY_NUM);
        cnode.write_slot(cap.get_raw_cap(), ROOT_CNODE_IDX);
        cap
    }
}

#[inline]
fn get_flags(flags: u32) -> usize {
    let ret = if ProgramFlags::is_executable(flags) {
        PAGE_X
    } else {
        0
    } | if ProgramFlags::is_writable(flags) {
        PAGE_W
    } else {
        0
    } | if ProgramFlags::is_readable(flags) {
        PAGE_R
    } else {
        0
    };
    ret
}

pub fn load_elf(process: &mut Process, elf_header: *const Elf64Hdr) {
    unsafe {
        for idx in 0..(*elf_header).e_phnum {
            let p_header = (*elf_header)
                .get_pheader(elf_header.cast::<usize>(), idx)
                .unwrap();
            if !((*p_header).p_type == ProgramType::Load) {
                continue;
            }
            let flags = get_flags((*p_header).p_flags) | PAGE_U;
            // this is start address of mapping segment
            let p_vaddr = VirtAddr::new((*p_header).p_vaddr);
            let p_start_addr = elf_header.cast::<u8>().add((*p_header).p_offset);
            // Sometime memsz > filesz, for example bss
            // so have to call copy with caring of this situation.
            let page_num = (align_up((*p_header).p_memsz, PAGE_SIZE)) / PAGE_SIZE;
            let mut file_sz_rem = (*p_header).p_filesz;

            for page_idx in 0..page_num {
                let page = alloc_vm().unwrap();
                if !(file_sz_rem == 0) {
                    let copy_src = p_start_addr.add(PAGE_SIZE * page_idx);
                    let copy_dst = page.addr as *mut u8;
                    let copy_size = min(PAGE_SIZE, file_sz_rem);
                    file_sz_rem = file_sz_rem.saturating_sub(PAGE_SIZE);
                    ptr::copy::<u8>(copy_src, copy_dst, copy_size);
                }
                process.map_page(p_vaddr.add(PAGE_SIZE * page_idx), page.into(), flags);
            }
        }
    }
}

pub struct MemoryRanges {}

impl MemoryRanges {
    pub fn alloc_size(&mut self, size: usize) -> KernelVAddress {
        todo!()
    }

    pub fn into_untyped(self) -> IntoIter<UntypedCap, 3> {
        todo!()
    }
}

fn create_initial_thread(memory_range: &mut MemoryRanges, root_server_mem: &mut RootServerMemory) -> () {
    // 1, create root cnode and insert self cap into self(root cnode)
    // 2, create vm space for root server,
    // 3, create root server tcb,
    // 4, convert rest of memory into untyped objects.
    let root_cnode_cap = root_server_mem.get_cnode_cap();

}
