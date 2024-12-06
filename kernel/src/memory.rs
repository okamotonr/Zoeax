use core::{ptr, marker::PhantomData, ops, fmt};
use crate::common::{KernelResult, Err};
use core::arch::naked_asm;

extern "C" {
    pub static __free_ram: u8;
    pub static __free_ram_end: u8;
    pub static __kernel_base: u8;
}

pub const PAGE_SIZE: usize = 4096;

// dummy init
static mut NEXT_PADDR: PhysAddr = PhysAddr::new(0);
static mut RAM_END: PhysAddr = PhysAddr::new(0);

pub unsafe fn copy_to_user<T: Sized>(src: VirtAddr, dst: VirtAddr) -> KernelResult<()> {
    let kernel_base = ptr::addr_of!(__kernel_base) as usize;
    // TODO:
    //if dst < PAGE_SIZE.into() || dst > kernel_base.into() {
    //    return Err(Err::InvalidUserAddress)
    //}
    //if dst + size_of::<T>().into() > kernel_base.into() {
    //    return Err(Err::InvalidUserAddress)
    //}
    mem_copy_to_user(dst.addr as *mut T, src.addr as *const T, size_of::<T>());
    Ok(())
}

pub unsafe fn copy_from_user<T: Sized>(addr: VirtAddr, dst: *mut T) -> KernelResult<()> {
    let kernel_base = ptr::addr_of!(__kernel_base) as usize;
    // TODO:
    // if addr < PAGE_SIZE.into() || addr > kernel_base.into() {
    //     return Err(Err::InvalidUserAddress)
    // }
    // if addr + size_of::<T>().into() > kernel_base.into() {
    //     return Err(Err::InvalidUserAddress)
    // }
    mem_copy_from_user(dst, addr.addr as *const T, size_of::<T>());
    Ok(())
}

#[naked]
extern "C" fn mem_copy_from_user<T>(dst: *mut T, src: *const T, len: usize) {
    unsafe {
        naked_asm!(
            "beqz a2, 2f",
            "1:",
            "lb a3, 0(a1)",
            "sb a3, 0(a0)",
            "addi a1, a1, 1",
            "addi a0, a0, 1",
            "addi a2, a2, -1",
            "bnez a2, 1b",
            "2:",
            "ret"
        )
    }
}

#[naked]
extern "C" fn mem_copy_to_user<T>(dst: *mut T, src: *const T, len: usize) {
    unsafe {
        naked_asm!(
            //".global riscv32_usercopy2",
            "beqz a2, 2f",        // a2 (コピー長) がゼロならラベル「2」にジャンプ
            "1:",
            "lb a3, 0(a1)",       // a1レジスタの指すアドレス (カーネルポインタ) から1バイト読み込む
            //"riscv32_usercopy2:",
            "sb a3, 0(a0)",       // a0レジスタの指すアドレス (ユーザーポインタ) に1バイト書き込む
            "addi a0, a0, 1",     // a0レジスタ (ユーザーポインタ) を1バイト進める
            "addi a1, a1, 1",     // a1レジスタ (カーネルポインタ) を1バイト進める
            "addi a2, a2, -1",    // a2レジスタの値を1減らす
            "bnez a2, 1b",        // a2レジスタの値がゼロでなければラベル「1」にジャンプ
            "2:",
            "ret"
        )
    }
}


#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address<T> {
    pub addr: usize,
    _phantom: PhantomData<fn() -> T>
}

impl<T> Address<T> {
    pub const fn new(addr: usize) -> Self {
        Self { addr, _phantom: PhantomData }
    }

    pub fn add(&self, count: usize) -> Self {
        Self { addr: self.addr + count, _phantom: self._phantom }
    }
}

impl<T> From<usize> for Address<T> {
    fn from(item: usize) -> Self {
        Self::new(item)
    }
}

impl<T> Into<usize> for Address<T> {
    fn into(self) -> usize {
        self.addr
    }
}

impl<T> fmt::Debug for Address<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address {{0x{:x}, {}}}:", self.addr, core::any::type_name::<T>())
    }
}

impl<T> ops::Add for Address<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { addr: self.addr + rhs.addr, _phantom: PhantomData }
    }
}

impl<T> ops::AddAssign for Address<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.addr += rhs.addr;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Physical;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Virtual;

pub type PhysAddr = Address<Physical>;
pub type VirtAddr = Address<Virtual>;

impl From<PhysAddr> for VirtAddr {
    fn from(value: PhysAddr) -> Self {
        Self::new(value.addr)
    }
}

pub fn init_memory() {
    unsafe {
        NEXT_PADDR = PhysAddr::new(ptr::addr_of!(__free_ram) as usize);
        RAM_END = PhysAddr::new(ptr::addr_of!(__free_ram_end) as usize);
    }
}

pub fn alloc_pages(n: usize) -> KernelResult<PhysAddr> {
    unsafe {
        let paddr = NEXT_PADDR;
        NEXT_PADDR += PhysAddr::new(n * PAGE_SIZE);

        if NEXT_PADDR.addr > RAM_END.addr {
            Err(Err::OutOfMemory)
        } else {
            ptr::write_bytes(paddr.addr as *mut u8, 0, n * PAGE_SIZE);
            Ok(paddr)
        }
    }
}
