use crate::{
    syscall::{
        cnode_copy, cnode_mint, configure_tcb, make_page_table_root, map_page, map_page_table,
        recv_ipc, recv_signal, resume_tcb, send_ipc, send_signal, set_ipc_buffer, unmap_page,
        untyped_retype, write_reg, SysCallFailed,
    },
    IPCBuffer,
};

use shared::{cap_type::CapabilityType, err_kind::ErrKind};
use shared::{registers::Registers, types::UntypedInfo};

pub trait KernelObject {
    const CAP_TYPE: CapabilityType;
    fn from_retype(user_size: usize, is_device: bool) -> Self;
}

pub trait FixedSizeObject: KernelObject {
    const OBJECT_SIZE: usize;
}

pub trait Copyable: KernelObject {
    fn copy_data(&self) -> Self;
}

pub trait Mintable: KernelObject {
    fn mint_data(&self, value: usize) -> Self;
}

#[derive(Debug)]
pub struct Capability<K: KernelObject> {
    // Or pub cnode: *mut CNode (or MutableCNodePtr, which gauarantee only one mutable ref is
    // existing)
    pub cap_ptr: usize,
    pub cap_depth: u32,
    pub cap_data: K,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Untyped {
    pub is_device: bool,
    pub size_bits: usize,
}

impl KernelObject for Untyped {
    const CAP_TYPE: CapabilityType = CapabilityType::Untyped;
    fn from_retype(user_size: usize, is_device: bool) -> Self {
        Self {
            size_bits: user_size,
            is_device,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CNode {
    pub radix: u32,
    // TODO: We have to track which slots are now in using.
    // Box<[Option<&Cap<Something>; 2_usize.pow(self.radix)]
    // or simple bitmap
    // Box<[bool; 2_usize.pow(self.radix)]
    pub cursor: usize,
}

impl KernelObject for CNode {
    const CAP_TYPE: CapabilityType = CapabilityType::CNode;
    fn from_retype(user_size: usize, _is_device: bool) -> Self {
        Self {
            radix: user_size as u32,
            cursor: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct PageTable {
    pub mapped_address: usize,
    pub is_root: bool,
    pub is_mapped: bool,
}

impl KernelObject for PageTable {
    const CAP_TYPE: CapabilityType = CapabilityType::PageTable;
    fn from_retype(_user_size: usize, _is_device: bool) -> Self {
        Self {
            mapped_address: 0,
            is_root: false,
            is_mapped: false,
        }
    }
}

impl FixedSizeObject for PageTable {
    const OBJECT_SIZE: usize = 4096;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Page {
    pub mapped_address: usize,
    pub is_mapped: bool,
    pub rights: PageFlags,
}

impl KernelObject for Page {
    const CAP_TYPE: CapabilityType = CapabilityType::Page;
    fn from_retype(_user_size: usize, _is_device: bool) -> Self {
        Self {
            mapped_address: 0,
            is_mapped: false,
            rights: PageFlags::never(),
        }
    }
}

impl FixedSizeObject for Page {
    const OBJECT_SIZE: usize = 4096;
}

impl Copyable for Page {
    fn copy_data(&self) -> Self {
        *self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Endpoint {}

impl KernelObject for Endpoint {
    const CAP_TYPE: CapabilityType = CapabilityType::EndPoint;
    fn from_retype(_user_size: usize, _is_device: bool) -> Self {
        Self {}
    }
}

impl FixedSizeObject for Endpoint {
    const OBJECT_SIZE: usize = 0;
}

impl Mintable for Endpoint {
    fn mint_data(&self, _value: usize) -> Self {
        *self
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Notificaiton {}

impl KernelObject for Notificaiton {
    const CAP_TYPE: CapabilityType = CapabilityType::Notification;
    fn from_retype(_user_size: usize, _is_device: bool) -> Self {
        Self {}
    }
}

impl FixedSizeObject for Notificaiton {
    const OBJECT_SIZE: usize = 0;
}

impl Copyable for Notificaiton {
    fn copy_data(&self) -> Self {
        *self
    }
}

impl Mintable for Notificaiton {
    fn mint_data(&self, _value: usize) -> Self {
        *self
    }
}

#[derive(Debug, Default)]
pub struct ThreadControlBlock {}

impl KernelObject for ThreadControlBlock {
    const CAP_TYPE: CapabilityType = CapabilityType::Tcb;
    fn from_retype(_user_size: usize, _is_device: bool) -> Self {
        Self {}
    }
}

impl FixedSizeObject for ThreadControlBlock {
    // Dummy
    const OBJECT_SIZE: usize = 0;
}

pub type UntypedCapability = Capability<Untyped>;

impl UntypedCapability {
    // only from Untyped info in BootInfo
    pub fn from_untyped_info(root_radix: u32, info: &UntypedInfo) -> Self {
        Self {
            cap_ptr: info.idx,
            cap_depth: root_radix,
            cap_data: Untyped {
                is_device: info.is_device,
                size_bits: info.bits,
            },
        }
    }

    pub fn retype_single<T: KernelObject>(
        &mut self,
        slot: &mut CSlot,
        user_size: usize,
    ) -> Result<Capability<T>, SysCallFailed> {
        let num = 1;
        untyped_retype(
            self.cap_ptr,
            self.cap_depth,
            slot.pptr,
            slot.depth,
            slot.index,
            user_size as u32,
            num,
            T::CAP_TYPE,
        )?;
        let new_c = T::from_retype(user_size, self.cap_data.is_device);
        // We have to caluculate new cap postion.
        let (cap_ptr, cap_depth) = slot.get_cap_ptr();
        Ok(Capability {
            cap_ptr,
            cap_depth,
            cap_data: new_c,
        })
    }

    pub fn retype_single_with_fixed_size<T: FixedSizeObject>(
        &mut self,
        slot: &mut CSlot,
    ) -> Result<Capability<T>, SysCallFailed> {
        // NOTE: user_size will be ignored in kernel.
        let user_size = T::OBJECT_SIZE;
        self.retype_single::<T>(slot, user_size)
    }
}

#[derive(Debug)]
pub struct CSlot {
    // Parent CNode path
    pptr: usize,
    depth: u32,
    // Parent radix
    radix: u32,
    index: u32,
}

impl CSlot {
    pub fn get_cap_ptr(&self) -> (usize, u32) {
        // TODO: check overflow
        // TODO: If this is from root cnode, we don't have to add
        let new_depth = self.depth + self.radix;
        let new_pptr = (self.pptr << self.radix) + self.index as usize;
        (new_pptr, new_depth)
    }
}

pub type CNodeCapability = Capability<CNode>;

impl CNodeCapability {
    pub fn get_slot(&mut self) -> Result<CSlot, SysCallFailed> {
        let size = self.get_size();
        if self.cap_data.cursor >= size {
            Err((ErrKind::NoEnoughSlot, 0))
        } else {
            let ret = Ok(CSlot {
                pptr: self.cap_ptr,
                depth: self.cap_depth,
                radix: self.cap_data.radix,
                index: self.cap_data.cursor as u32,
            });
            self.cap_data.cursor += 1;
            ret
        }
    }

    pub fn get_size(&self) -> usize {
        2_usize.pow(self.cap_data.radix)
    }

    pub fn copy<K: KernelObject + Copyable>(
        &mut self,
        cap: &Capability<K>,
    ) -> Result<Capability<K>, SysCallFailed> {
        let slot = self.get_slot()?;
        let depth = slot.radix;
        let index = slot.index as usize;
        let (dest_ptr, dest_depth) = slot.get_cap_ptr();
        cnode_copy(
            self.cap_ptr,
            self.cap_depth,
            index,
            depth,
            cap.cap_ptr,
            cap.cap_depth,
        )?;
        let cap_data = cap.cap_data.copy_data();
        Ok(Capability {
            cap_ptr: dest_ptr,
            cap_depth: dest_depth,
            cap_data,
        })
    }

    pub fn delete<K: KernelObject>(
        &mut self,
        mut _cap: Capability<K>,
    ) -> Result<(), SysCallFailed> {
        // kernel adaptation is not yet done.
        Ok(())
    }

    pub fn mint<K: KernelObject + Mintable>(
        &mut self,
        cap: &Capability<K>,
        cap_val: usize,
    ) -> Result<Capability<K>, SysCallFailed> {
        let (dest_ptr, dest_depth) = self.get_slot()?.get_cap_ptr();
        cnode_mint(
            self.cap_ptr,
            self.cap_depth,
            dest_ptr,
            dest_depth,
            cap.cap_ptr,
            cap.cap_depth,
            cap_val,
        )?;
        let cap_data = K::mint_data(&cap.cap_data, cap_val);
        Ok(Capability {
            cap_ptr: dest_ptr,
            cap_depth: dest_depth,
            cap_data,
        })
    }
}

pub type PageTableCapability = Capability<PageTable>;

impl PageTableCapability {
    pub fn map(&mut self, root_table: &mut Self, vaddr: usize) -> Result<usize, SysCallFailed> {
        map_page_table(
            self.cap_ptr,
            self.cap_depth,
            root_table.cap_ptr,
            root_table.cap_depth,
            vaddr,
        )
    }

    pub fn make_as_root(&mut self) -> Result<(), SysCallFailed> {
        make_page_table_root(self.cap_ptr, self.cap_depth)?;
        self.cap_data.is_mapped = true;
        Ok(())
    }
}

pub type PageCapability = Capability<Page>;

impl PageCapability {
    pub fn map(
        &mut self,
        root_table: &mut PageTableCapability,
        vaddr: usize,
        flags: PageFlags,
    ) -> Result<(), SysCallFailed> {
        map_page(
            self.cap_ptr,
            self.cap_depth,
            root_table.cap_ptr,
            root_table.cap_depth,
            vaddr,
            flags.into(),
        )?;
        self.cap_data.is_mapped = true;
        self.cap_data.mapped_address = vaddr;
        self.cap_data.rights = flags;
        Ok(())
    }

    pub fn unmap(&mut self, root_table: &mut PageTableCapability) -> Result<(), SysCallFailed> {
        unmap_page(
            self.cap_ptr,
            self.cap_depth,
            root_table.cap_ptr,
            root_table.cap_depth,
        )?;
        self.cap_data.is_mapped = false;
        self.cap_data.mapped_address = 0;
        self.cap_data.rights = PageFlags::never();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PageFlags {
    pub is_writable: bool,
    pub is_readable: bool,
    pub is_executable: bool,
}

impl From<PageFlags> for usize {
    fn from(value: PageFlags) -> Self {
        let write = if value.is_writable { 0x02 } else { 0x00 };
        let read = if value.is_readable { 0x04 } else { 0x00 };
        let exec = if value.is_executable { 0x01 } else { 0x00 };
        write | read | exec
    }
}

impl PageFlags {
    pub fn readonly() -> Self {
        Self {
            is_writable: false,
            is_readable: true,
            is_executable: false,
        }
    }

    pub fn writeonly() -> Self {
        Self {
            is_writable: true,
            is_readable: false,
            is_executable: false,
        }
    }

    pub fn executable() -> Self {
        Self {
            is_writable: false,
            is_readable: true,
            is_executable: true,
        }
    }

    pub fn all() -> Self {
        Self {
            is_writable: true,
            is_readable: true,
            is_executable: true,
        }
    }

    pub fn readandwrite() -> Self {
        Self {
            is_writable: true,
            is_readable: true,
            is_executable: false,
        }
    }

    pub fn never() -> Self {
        Self {
            is_writable: false,
            is_readable: false,
            is_executable: false,
        }
    }
}

pub type TCBCapability = Capability<ThreadControlBlock>;

impl TCBCapability {
    pub fn set_ipc_buffer(&mut self, page_cap: &PageCapability) -> Result<(), SysCallFailed> {
        set_ipc_buffer(
            self.cap_ptr,
            self.cap_depth,
            page_cap.cap_ptr,
            page_cap.cap_depth,
        )?;
        Ok(())
    }

    pub fn write_regs<F: FnOnce() -> Registers>(
        &mut self,
        writer: F,
        ipc_buffer: &mut IPCBuffer,
    ) -> Result<(), SysCallFailed> {
        write_reg(self.cap_ptr, self.cap_depth, writer, ipc_buffer)?;
        Ok(())
    }

    pub fn configure(
        &mut self,
        root_cnode: &mut CNodeCapability,
        root_vspace: &mut PageTableCapability,
    ) -> Result<(), SysCallFailed> {
        configure_tcb(
            self.cap_ptr,
            self.cap_depth,
            root_cnode.cap_ptr,
            root_cnode.cap_depth,
            root_vspace.cap_ptr,
            root_vspace.cap_depth,
        )?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), SysCallFailed> {
        resume_tcb(self.cap_ptr, self.cap_depth)?;
        Ok(())
    }
}

pub type EndpointCapability = Capability<Endpoint>;

impl EndpointCapability {
    pub fn send(&self) -> Result<(), SysCallFailed> {
        send_ipc(self.cap_ptr, self.cap_depth)?;
        Ok(())
    }

    pub fn recive(&self) -> Result<usize, SysCallFailed> {
        recv_ipc(self.cap_ptr, self.cap_depth)
    }
}

pub type NotificaitonCapability = Capability<Notificaiton>;

impl NotificaitonCapability {
    pub fn send(&self) -> Result<(), SysCallFailed> {
        send_signal(self.cap_ptr, self.cap_depth)?;
        Ok(())
    }

    pub fn wait(&self) -> Result<usize, SysCallFailed> {
        recv_signal(self.cap_ptr, self.cap_depth)
    }
}
