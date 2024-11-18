use core::{ptr, marker::PhantomData, ops, fmt};

extern "C" {
    static __free_ram: u8;
    pub static __free_ram_end: u8;
}

pub const PAGE_SIZE: usize = 4096;

// dummy init
static mut NEXT_PADDR: PhysAddr = PhysAddr::new(0);
static mut RAM_END: PhysAddr = PhysAddr::new(0);


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

pub fn alloc_pages(n: usize) -> PhysAddr {
    unsafe {
        let paddr = NEXT_PADDR;
        NEXT_PADDR += PhysAddr::new(n * PAGE_SIZE);

        if NEXT_PADDR.addr > RAM_END.addr {
            panic!("out of memory")
        };
        ptr::write_bytes(paddr.addr as *mut u8, 0, n * PAGE_SIZE);
        paddr
    }
}
