use super::pm::BumpAllocator;
use crate::elf::{Elf64Hdr, Elf64Phdr, ProgramFlags, ProgramType};

use crate::address::KernelVAddress;
use crate::address::VirtAddr;
use crate::address::PAGE_SIZE;
use crate::capability::cnode::CNodeCap;
use crate::capability::page_table::PageCap;
use crate::capability::page_table::PageTableCap;
use crate::capability::tcb::TCBCap;
use crate::capability::untyped::UntypedCap;
use crate::capability::Capability;
use crate::capability::CapabilityData;
use crate::capability::Something;
use crate::common::{align_up, BootInfo, ErrKind, UntypedInfo};
use crate::object::page_table::{Page, PAGE_R, PAGE_U, PAGE_W, PAGE_X};
use crate::object::CNode;
use crate::object::CNodeEntry;
use crate::object::PageTable;
use crate::object::ThreadControlBlock;
use crate::object::ThreadInfo;
use crate::println;

use crate::riscv::SSTATUS_SPIE;
use crate::scheduler::create_idle_thread;
use crate::scheduler::require_schedule;
use crate::scheduler::schedule;
use core::cmp::min;
use core::mem::MaybeUninit;
use core::ptr;

// TODO
const ROOT_CNODE_ENTRY_NUM_BITS: usize = 18; // 2^18
const ROOT_TCB_IDX: usize = 1;
const ROOT_CNODE_IDX: usize = 2;
const ROOT_VSPACE_IDX: usize = 3;
const ROOT_IPC_BUFFER: usize = 4;
const ROOT_BOOT_INFO_PAGE: usize = 5;

extern "C" {
    static __stack_top: u8;
}

impl CNode {
    // todo: broken
    fn write_slot(&mut self, cap: CapabilityData<Something>, index: usize) {
        let root = (self as *mut Self).cast::<Option<CNodeEntry<Something>>>();
        let entry = CNodeEntry::new_with_rawcap(cap);
        assert!(unsafe { (*root.add(index)).is_none() });
        unsafe { *root.add(index) = Some(entry) }
    }
}

impl CNodeCap {
    fn write_slot(&mut self, cap: CapabilityData<Something>, index: usize) {
        let cnode = self.get_cnode();
        let entry = CNodeEntry::new_with_rawcap(cap);
        assert!(cnode[index].is_none());
        cnode[index] = Some(entry);
    }
}

struct RootServerMemory<'a> {
    cnode: &'a mut MaybeUninit<CNode>,
    vspace: &'a mut MaybeUninit<PageTable>,
    tcb: &'a mut MaybeUninit<ThreadControlBlock>,
    ipc_buf: &'a mut MaybeUninit<Page>,
    boot_frame: &'a mut MaybeUninit<Page>,
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
        let cnode = Self::alloc_obj::<CNodeCap>(bump_allocator, ROOT_CNODE_ENTRY_NUM_BITS);
        let vspace = Self::alloc_obj::<PageTableCap>(bump_allocator, 0);
        let tcb = Self::alloc_obj::<TCBCap>(bump_allocator, 0);
        let ipc_buf = Self::alloc_obj::<PageCap>(bump_allocator, 0);
        let boot_frame = Self::alloc_obj::<PageCap>(bump_allocator, 0);
        Self {
            cnode,
            vspace,
            tcb,
            ipc_buf,
            boot_frame,
        }
    }

    fn create_root_cnode(&mut self) -> CNodeCap {
        let cnode = self.cnode.write(CNode::new());
        let vaddr = (cnode as *const CNode).into();
        let cap_dep_val = CNodeCap::create_cap_dep_val(vaddr, ROOT_CNODE_ENTRY_NUM_BITS);
        let cap_type = CNodeCap::CAP_TYPE;
        let cap = CNodeCap::new(cap_type, vaddr.into(), cap_dep_val as u64);
        let cap_in_slot = cap.replicate().up_cast();
        cnode.write_slot(cap_in_slot, ROOT_CNODE_IDX);
        cap
    }

    /// create address space of initial server.
    fn create_address_space(
        &mut self,
        cnode_cap: &mut CNodeCap,
        elf_header: *const Elf64Hdr,
        bootstage_mbr: &mut BootStateManager,
    ) -> (PageTableCap, VirtAddr) {
        let root_page_table = self.vspace.write(PageTable::new());

        let mut max_vaddr = VirtAddr::new(0);
        root_page_table.copy_global_mapping();
        let vaddr = (root_page_table as *const PageTable).into();
        let mut cap = PageTableCap::init(vaddr, 0);
        cap.root_map().unwrap();
        cnode_cap.write_slot(cap.replicate().up_cast(), ROOT_VSPACE_IDX);
        unsafe {
            for idx in 0..(*elf_header).e_phnum {
                let p_header = (*elf_header)
                    .get_pheader(elf_header.cast::<usize>(), idx)
                    .unwrap();
                let p_start_addr = elf_header.cast::<u8>().add((*p_header).p_offset);
                allocate_p_segment(
                    cnode_cap,
                    &mut cap,
                    bootstage_mbr,
                    p_header,
                    p_start_addr,
                    &mut max_vaddr,
                )
            }
        }
        (cap, max_vaddr)
    }

    /// create ipc buffer frame
    fn create_ipc_buf_frame(
        &mut self,
        cnode_cap: &mut CNodeCap,
        vspace_cap: &mut PageTableCap,
        max_vaddr: VirtAddr,
        bootstage_mbr: &mut BootStateManager,
    ) -> PageCap {
        let ipc_buf_frame = self.ipc_buf.write(Page::new());
        let vaddr = (ipc_buf_frame as *const Page).into();
        let flags = PAGE_R | PAGE_W | PAGE_U;
        let page_cap = create_mapped_page_cap(
            cnode_cap,
            vspace_cap,
            bootstage_mbr,
            vaddr,
            max_vaddr.add(PAGE_SIZE),
            flags,
        );
        cnode_cap.write_slot(page_cap.up_cast(), ROOT_IPC_BUFFER);
        page_cap
    }

    fn create_boot_info_frame(
        &mut self,
        cnode_cap: &mut CNodeCap,
        vspace_cap: &mut PageTableCap,
        max_vaddr: VirtAddr,
        bootstage_mbr: &mut BootStateManager,
    ) -> (PageCap, KernelVAddress) {
        let boot_page = self.boot_frame.write(Page::new());
        let vaddr = (boot_page as *const Page).into();
        let flags = PAGE_R | PAGE_U;
        let page_cap = create_mapped_page_cap(
            cnode_cap,
            vspace_cap,
            bootstage_mbr,
            vaddr,
            max_vaddr.add(PAGE_SIZE),
            flags,
        );
        cnode_cap.write_slot(page_cap.up_cast(), ROOT_BOOT_INFO_PAGE);
        (page_cap, vaddr)
    }

    fn create_root_tcb(
        &mut self,
        cnode_cap: &mut CNodeCap,
        vspace_cap: &mut PageTableCap,
        ipc_buf_cap: &mut PageCap,
        entry_point: VirtAddr,
    ) -> TCBCap {
        let tcb = self
            .tcb
            .write(ThreadControlBlock::new(ThreadInfo::default()));
        // TODO: per arch
        tcb.registers.sstatus = SSTATUS_SPIE;
        tcb.registers.sepc = entry_point.into();

        // insert cnode_cap into tcb cnode_cap
        let mut new_entry = CNodeEntry::new_with_rawcap(cnode_cap.replicate());
        new_entry.insert(
            cnode_cap
                .lookup_entry_mut_one_level(ROOT_CNODE_IDX)
                .unwrap()
                .as_mut()
                .unwrap(),
        );
        tcb.root_cnode = Some(new_entry);
        // insert vspace cap into tcb vspace
        let mut new_entry = CNodeEntry::new_with_rawcap(vspace_cap.replicate());
        new_entry.insert(
            cnode_cap
                .lookup_entry_mut_one_level(ROOT_VSPACE_IDX)
                .unwrap()
                .as_mut()
                .unwrap(),
        );
        tcb.vspace = Some(new_entry);
        let mut new_entry = CNodeEntry::new_with_rawcap(ipc_buf_cap.replicate());
        let vaddr = ipc_buf_cap.get_address();
        let mapped = ipc_buf_cap.get_mapped_address();
        new_entry.insert(
            cnode_cap
                .lookup_entry_mut_one_level(ROOT_IPC_BUFFER)
                .unwrap()
                .as_mut()
                .unwrap(),
        );
        tcb.ipc_buffer = Some(new_entry);

        let cap = TCBCap::init((tcb as *const ThreadControlBlock).into(), 0);
        cnode_cap.write_slot(cap.replicate().up_cast(), ROOT_TCB_IDX);
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

    pub fn finalize(self) -> UntypedCapGenerator {
        let (start_address, end_address) = self.bump_allocator.end_allocation();
        UntypedCapGenerator {
            start_address: start_address.into(),
            end_address: end_address.into(),
            idx_start: self.cnode_satrt_idx,
            idx_max: self.cnode_idx_max,
        }
    }
}

struct UntypedCapGenerator {
    start_address: KernelVAddress,
    end_address: KernelVAddress,
    idx_start: usize,
    idx_max: usize,
}

impl Iterator for UntypedCapGenerator {
    type Item = (usize, UntypedCap);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_address.add(4096) >= self.end_address {
            return None;
        }
        assert!(self.idx_max >= self.idx_start);
        let block_size: usize = (self.end_address - self.start_address).into();
        let untyped_cap = UntypedCap::init(self.start_address.into(), block_size);
        let acctual_size = untyped_cap.block_size();
        self.start_address = self.start_address.add(acctual_size);
        let ret = Some((self.idx_start, untyped_cap));
        self.idx_start += 1;
        ret
    }
}

unsafe fn allocate_p_segment(
    cnode_cap: &mut CNodeCap,
    root_table_cap: &mut PageTableCap,
    bootstage_mbr: &mut BootStateManager,
    p_header: *const Elf64Phdr,
    p_start_addr: *const u8,
    max_vaddr: &mut VirtAddr,
) {
    if !((*p_header).p_type == ProgramType::Load) {
        return;
    }
    let flags = get_flags((*p_header).p_flags) | PAGE_U;
    let vaddr = VirtAddr::new((*p_header).p_vaddr);
    let page_num = (align_up((*p_header).p_memsz, PAGE_SIZE)) / PAGE_SIZE;
    let mut file_sz_rem = (*p_header).p_filesz;
    for page_idx in 0..page_num {
        let vaddr_n = vaddr.add(PAGE_SIZE * page_idx);
        if *max_vaddr < vaddr_n {
            *max_vaddr = vaddr_n;
        }
        let page_addr = bootstage_mbr.alloc_page();
        let page_cap = create_mapped_page_cap(
            cnode_cap,
            root_table_cap,
            bootstage_mbr,
            page_addr,
            vaddr_n,
            flags,
        );
        cnode_cap.write_slot(page_cap.up_cast(), bootstage_mbr.alloc_cnode_idx());
        if file_sz_rem != 0 {
            let copy_src = p_start_addr.add(PAGE_SIZE * page_idx);
            let copy_dst = page_cap.get_address_virtual().addr as *mut u8;
            let copy_size = min(PAGE_SIZE, file_sz_rem);
            file_sz_rem = file_sz_rem.saturating_sub(PAGE_SIZE);
            ptr::copy::<u8>(copy_src, copy_dst, copy_size);
        }
    }
}

fn create_mapped_page_cap(
    cnode_cap: &mut CNodeCap,
    root_table_cap: &mut PageTableCap,
    bootstage_mbr: &mut BootStateManager,
    paddr: KernelVAddress,
    vaddr_n: VirtAddr,
    flags: usize,
) -> PageCap {
    let mut page_cap = PageCap::init(paddr, 0);
    if let Err(e) = page_cap.map(root_table_cap, vaddr_n, flags) {
        match e.e_kind {
            ErrKind::PageTableNotMappedYet => {
                map_page_tables(cnode_cap, bootstage_mbr, root_table_cap, vaddr_n);
                page_cap.map(root_table_cap, vaddr_n, flags).unwrap();
            }
            ErrKind::VaddressAlreadyMapped => {
                panic!("Should never occur")
            }
            _ => {
                panic!("Unknown Error occured")
            }
        }
    };
    page_cap
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
            page_table_cap.replicate().up_cast(),
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

fn create_initial_thread(
    root_server_mem: &mut RootServerMemory,
    mut bootstage_mbr: BootStateManager,
    elf_header: *const Elf64Hdr,
) {
    // 8, call return_to_user(after returning user, to clear stack)
    // 1, create root cnode and insert self cap into self(root cnode)
    let mut root_cnode_cap = root_server_mem.create_root_cnode();
    // 2, create vm space for root server,
    let (mut vspace_cap, max_vaddr) =
        root_server_mem.create_address_space(&mut root_cnode_cap, elf_header, &mut bootstage_mbr);
    // 3, create ipc buffer frame
    let mut ipc_page_cap = root_server_mem.create_ipc_buf_frame(
        &mut root_cnode_cap,
        &mut vspace_cap,
        max_vaddr,
        &mut bootstage_mbr,
    );

    let (_, boot_info_addr) = root_server_mem.create_boot_info_frame(
        &mut root_cnode_cap,
        &mut vspace_cap,
        max_vaddr.add(PAGE_SIZE),
        &mut bootstage_mbr,
    );
    // 4, create idle thread
    create_idle_thread(&raw const __stack_top as usize);
    // 5, create root server tcb,
    let boot_info_ptr: *mut BootInfo = boot_info_addr.into();
    let boot_info = unsafe {
        *boot_info_ptr = BootInfo::default();
        boot_info_ptr.as_mut().unwrap()
    };

    let entry_point = unsafe { (*elf_header).e_entry };
    let mut root_tcb = root_server_mem.create_root_tcb(
        &mut root_cnode_cap,
        &mut vspace_cap,
        &mut ipc_page_cap,
        entry_point.into(),
    );

    // 6, convert rest of memory into untyped objects.
    let mut num = 0;
    for (idx, (untyped_cap_idx, untyped_cap)) in bootstage_mbr.finalize().enumerate() {
        assert!(num < 32);
        root_cnode_cap.write_slot(untyped_cap.replicate().up_cast(), untyped_cap_idx);
        boot_info.untyped_infos[idx] = UntypedInfo {
            bits: untyped_cap.block_size(),
            idx: untyped_cap_idx,
            is_device: false,
        };
        num += 1;
        boot_info.firtst_empty_idx = untyped_cap_idx + 1;
    }
    boot_info.untyped_num = num;
    for (i, ch) in "hello, root_server\n".as_bytes().iter().enumerate() {
        boot_info.msg[i] = *ch;
    }
    boot_info.root_cnode_idx = ROOT_CNODE_IDX;
    boot_info.root_vspace_idx = ROOT_VSPACE_IDX;
    boot_info.ipc_buffer_addr = max_vaddr.add(PAGE_SIZE).into();
    // 7, set initial thread into current thread
    root_tcb.set_register(&[(10, max_vaddr.add(PAGE_SIZE * 2).into())]);
    root_tcb.make_runnable();
    println!("root process initialization finished");
}

pub fn init_root_server(mut bump_allocator: BumpAllocator, elf_header: *const Elf64Hdr) {
    let mut root_server_mem = RootServerMemory::init_with_uninit(&mut bump_allocator);
    let bootstage_mbr = BootStateManager::new(
        bump_allocator,
        ROOT_BOOT_INFO_PAGE + 1,
        2_usize.pow(ROOT_CNODE_ENTRY_NUM_BITS as u32) - 1,
    );
    create_initial_thread(&mut root_server_mem, bootstage_mbr, elf_header);

    require_schedule();
    unsafe {
        schedule();
    }
}
