use super::pm::BumpAllocator;
use common::elf::{Elf64Hdr, Elf64Phdr, ProgramFlags, ProgramType};

use crate::address::VirtAddr;
use crate::address::PAGE_SIZE;
use crate::capability::cnode::CNodeCap;
use crate::capability::page_table::PageCap;
use crate::capability::page_table::PageTableCap;
use crate::capability::tcb::TCBCap;
use crate::capability::untyped::UntypedCap;
use crate::capability::Capability;
use crate::capability::RawCapability;
use crate::common::{align_up, Err};
use crate::object::CNode;
use crate::object::CNodeEntry;
use crate::object::PageTable;
use crate::object::ThreadControlBlock;
use crate::object::ThreadInfo;
use crate::println;

use crate::riscv::SSTATUS_SPIE;
use crate::riscv::SSTATUS_SUM;
use crate::scheduler::create_idle_thread;
use crate::scheduler::schedule;
use crate::vm::KernelVAddress;
use crate::vm::{PAGE_R, PAGE_U, PAGE_W, PAGE_X};
use core::cmp::min;
use core::mem::MaybeUninit;
use core::ptr;

// TODO
const ROOT_CNODE_ENTRY_NUM: usize = 64;
const ROOT_TCB_IDX: usize = 1;
const ROOT_CNODE_IDX: usize = 2;
const ROOT_VSPACE_IDX: usize = 3;

// Only initalization
impl CNode {
    fn write_slot(&mut self, cap: RawCapability, index: usize) {
        println!("{:?}", cap.get_cap_type());
        let root = (self as *mut Self).cast::<CNodeEntry>();
        let entry = CNodeEntry::new_with_rawcap(cap);
        unsafe { *root.add(index) = entry }
    }
}

impl CNodeCap {
    fn write_slot(&mut self, cap: RawCapability, index: usize) {
        println!("{:?}", self.get_raw_cap().get_address());
        println!("{:?}", self.get_raw_cap().get_cap_type());
        let cnode = self.get_cnode(1, index).unwrap();
        println!("{:p}", cnode);
        cnode.write_slot(cap, index);
    }
}

struct RootServerMemory<'a> {
    cnode: &'a mut MaybeUninit<CNode>,
    vspace: &'a mut MaybeUninit<PageTable>,
    tcb: &'a mut MaybeUninit<ThreadControlBlock>,
}

impl<'a> RootServerMemory<'a> {
    fn alloc_obj<T>(
        bump_allocator: &mut BumpAllocator,
        user_size: usize,
    ) -> &'a mut MaybeUninit<T::KernelObject>
    where
        T: Capability,
    {
        // easiest way to care align.
        let start_address = bump_allocator
            .allocate_pages(align_up(T::get_object_size(user_size), PAGE_SIZE) / PAGE_SIZE)
            .into();
        let cnode_ptr = <KernelVAddress as Into<*mut T::KernelObject>>::into(start_address);
        unsafe { cnode_ptr.as_uninit_mut().unwrap() }
    }

    fn init_with_uninit(bump_allocator: &mut BumpAllocator) -> Self {
        let cnode = Self::alloc_obj::<CNodeCap>(bump_allocator, ROOT_CNODE_ENTRY_NUM);
        let vspace = Self::alloc_obj::<PageTableCap>(bump_allocator, 0);
        let tcb = Self::alloc_obj::<TCBCap>(bump_allocator, 0);
        Self { cnode, vspace, tcb }
    }

    fn create_root_cnode(&mut self) -> CNodeCap {
        let cnode = self.cnode.write(CNode::new());
        let cap = CNodeCap::init((cnode as *const CNode).into(), ROOT_CNODE_ENTRY_NUM);
        cnode.write_slot(cap.get_raw_cap(), ROOT_CNODE_IDX);
        cap
    }

    /// create address space of initial server.
    fn create_address_space(
        &mut self,
        cnode_cap: &mut CNodeCap,
        elf_header: *const Elf64Hdr,
        bootstage_mbr: &mut BootStateManager,
    ) -> PageTableCap {
        let root_page_table = self.vspace.write(PageTable::new());

        root_page_table.copy_global_mapping();
        let mut cap = PageTableCap::init((root_page_table as *const PageTable).into(), 0);
        cap.root_map().unwrap();
        println!("Here");
        cnode_cap.write_slot(cap.get_raw_cap(), ROOT_VSPACE_IDX);
        unsafe {
            for idx in 0..(*elf_header).e_phnum {
                let p_header = (*elf_header)
                    .get_pheader(elf_header.cast::<usize>(), idx)
                    .unwrap();
                let p_start_addr = elf_header.cast::<u8>().add((*p_header).p_offset);
                allocate_p_segment(cnode_cap, &mut cap, bootstage_mbr, p_header, p_start_addr)
            }
        }
        cap
    }

    fn create_root_tcb(
        &mut self,
        cnode_cap: &mut CNodeCap,
        vspace_cap: &mut PageTableCap,
        entry_point: VirtAddr,
    ) -> TCBCap {
        let tcb = self
            .tcb
            .write(ThreadControlBlock::new(ThreadInfo::default()));
        // TODO: per arch
        tcb.registers.sstatus = SSTATUS_SPIE | SSTATUS_SUM;
        tcb.registers.sepc = entry_point.into();

        // insert cnode_cap into tcb cnode_cap
        let raw_cap = cnode_cap.get_raw_cap();
        tcb.root_cnode
            .insert(cnode_cap.lookup_entry(ROOT_CNODE_IDX).unwrap(), raw_cap);
        // insert vspace cap into tcb vspace
        tcb.vspace.insert(
            cnode_cap.lookup_entry(ROOT_VSPACE_IDX).unwrap(),
            vspace_cap.get_raw_cap(),
        );

        let cap = TCBCap::init((tcb as *const ThreadControlBlock).into(), 0);
        cnode_cap.write_slot(cap.get_raw_cap(), ROOT_TCB_IDX);
        cap
    }
}

struct BootStateManager {
    bump_allocator: BumpAllocator,
    cnode_satrt_idx: usize,
    cnode_idx_max: usize,
}

impl BootStateManager {
    pub fn new(
        bump_allocator: BumpAllocator,
        cnode_start_idx: usize,
        cnode_idx_max: usize,
    ) -> Self {
        assert!(cnode_start_idx < cnode_idx_max);
        Self {
            bump_allocator,
            cnode_satrt_idx: cnode_start_idx,
            cnode_idx_max,
        }
    }

    pub fn alloc_page(&mut self) -> KernelVAddress {
        self.bump_allocator.allocate_page().into()
    }

    pub fn alloc_cnode_idx(&mut self) -> usize {
        let ret = self.cnode_satrt_idx;
        assert!(ret <= self.cnode_idx_max);
        self.cnode_satrt_idx += 1;
        ret
    }

    pub fn into_untyped(self) -> UntypedCap {
        let (start_address, end_address) = self.bump_allocator.end_allocation();
        let block_size = (end_address - start_address).into();
        UntypedCap::init(start_address.into(), block_size)
    }
}

unsafe fn allocate_p_segment(
    cnode_cap: &mut CNodeCap,
    root_table_cap: &mut PageTableCap,
    bootstage_mbr: &mut BootStateManager,
    p_header: *const Elf64Phdr,
    p_start_addr: *const u8,
) {
    if !((*p_header).p_type == ProgramType::Load) {
        return;
    }
    let flags = get_flags((*p_header).p_flags) | PAGE_U;
    let vaddr = VirtAddr::new((*p_header).p_vaddr);
    let page_num = (align_up((*p_header).p_memsz, PAGE_SIZE)) / PAGE_SIZE;
    let mut file_sz_rem = (*p_header).p_filesz;
    for page_idx in 0..page_num {
        let page_addr = bootstage_mbr.alloc_page();
        let mut page_cap = PageCap::init(page_addr, 0);
        let vaddr_n = vaddr.add(PAGE_SIZE * page_idx);
        if let Err(e) = page_cap.map(root_table_cap, vaddr_n, flags) {
            match e {
                Err::PageTableNotMappedYet => {
                    map_page_tables(cnode_cap, bootstage_mbr, root_table_cap, vaddr_n);
                    page_cap.map(root_table_cap, vaddr_n, flags).unwrap();
                }
                Err::VaddressAlreadyMapped => {
                    panic!("Should never occur")
                }
                _ => {
                    panic!("Unknown Error occured")
                }
            }
        };
        if file_sz_rem != 0 {
            let copy_src = p_start_addr.add(PAGE_SIZE * page_idx);
            let copy_dst = page_cap.get_address().addr as *mut u8;
            let copy_size = min(PAGE_SIZE, file_sz_rem);
            file_sz_rem = file_sz_rem.saturating_sub(PAGE_SIZE);
            ptr::copy::<u8>(copy_src, copy_dst, copy_size);
            cnode_cap.write_slot(page_cap.get_raw_cap(), bootstage_mbr.alloc_cnode_idx())
        }
    }
}

fn map_page_tables(
    cnode_cap: &mut CNodeCap,
    bootstage_mbr: &mut BootStateManager,
    root_table_cap: &mut PageTableCap,
    vaddr_n: VirtAddr,
) {
    loop {
        let mut page_table_cap = PageTableCap::init(bootstage_mbr.alloc_page(), 0);
        cnode_cap.write_slot(
            page_table_cap.get_raw_cap(),
            bootstage_mbr.alloc_cnode_idx(),
        );
        if let Ok(level) = page_table_cap.map(root_table_cap, vaddr_n) {
            if level == 0 {
                break;
            }
        } else {
            panic!("error occur")
        }
    }
}

#[inline]
fn get_flags(flags: u32) -> usize {
    (if ProgramFlags::is_executable(flags) {
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
    })
}

pub fn init_root_server(mut bump_allocator: BumpAllocator, elf_header: *const Elf64Hdr) {
    let mut root_server_mem = RootServerMemory::init_with_uninit(&mut bump_allocator);
    let bootstage_mbr = BootStateManager::new(
        bump_allocator,
        ROOT_VSPACE_IDX + 1,
        ROOT_CNODE_ENTRY_NUM - 1,
    );
    create_initial_thread(&mut root_server_mem, bootstage_mbr, elf_header);

    unsafe {
        schedule();
    }
}

fn create_initial_thread(
    root_server_mem: &mut RootServerMemory,
    mut bootstage_mbr: BootStateManager,
    elf_header: *const Elf64Hdr,
) {
    // 8, call return_to_user(after returning user, to clear stack)
    // 1, create root cnode and insert self cap into self(root cnode)
    let mut root_cnode_cap = root_server_mem.create_root_cnode();
    // 2, create vm space for root server,
    let mut vspace_cap =
        root_server_mem.create_address_space(&mut root_cnode_cap, elf_header, &mut bootstage_mbr);
    // 3, create idle thread
    create_idle_thread();
    // 4, create root server tcb,
    let entry_point = unsafe { (*elf_header).e_entry };
    let mut root_tcb =
        root_server_mem.create_root_tcb(&mut root_cnode_cap, &mut vspace_cap, entry_point.into());

    // 6, convert rest of memory into untyped objects.
    let untyped_cap_idx = bootstage_mbr.alloc_cnode_idx();
    let untyped_cap = bootstage_mbr.into_untyped();
    root_cnode_cap.write_slot(untyped_cap.get_raw_cap(), untyped_cap_idx);
    // 7, set initial thread into current thread
    root_tcb.make_runnable();
    println!("root process initialization finished");
}
