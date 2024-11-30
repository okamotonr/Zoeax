use core::{
    mem, ptr, sync::atomic::{compiler_fence, Ordering::SeqCst}
};

use crate::{
    memory::{alloc_pages, PAGE_SIZE},
    println,
    common::align_up
};

pub const SECTOR_SIZE: u32 = 512;
const VIRTQ_ENTRY_NUM: usize = 16;
const VIRTIO_DEVICE_BLK: u32 = 2;
pub const VIRTIO_BLK_PADDR: usize = 0x10001000;
const VIRTIO_REG_MAGIC: usize = 0x00;
const VIRTIO_REG_VERSION: usize = 0x04;
const VIRTIO_REG_DEVICE_ID: usize = 0x08;
const VIRTIO_REG_QUEUE_SEL: usize = 0x30;
const VIRTIO_REG_QUEUE_NUM_MAX: usize = 0x34;
const VIRTIO_REG_QUEUE_NUM: usize = 0x38;
const VIRTIO_REG_QUEUE_ALIGN: usize = 0x3c;
const VIRTIO_REG_QUEUE_PFN: usize = 0x40;
const VIRTIO_REG_QUEUE_READY: usize = 0x44;
const VIRTIO_REG_QUEUE_NOTIFY: usize = 0x50;
const VIRTIO_REG_DEVICE_STATUS: usize = 0x70;
const VIRTIO_REG_DEVICE_CONFIG: usize = 0x100;
const VIRTIO_STATUS_ACK: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
const VIRTIO_STATUS_FEAT_OK: u32 = 8;
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const VIRTQ_AVAIL_F_NO_INTERRUPT: usize = 1;
const VIRTIO_BLK_T_IN: usize = 0;
const VIRTIO_BLK_T_OUT: usize = 1;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqDesc {
    addr: usize,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; VIRTQ_ENTRY_NUM],
    used_event: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; VIRTQ_ENTRY_NUM],
    avail_event: u16,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct Virtq {
    desc: [VirtqDesc; VIRTQ_ENTRY_NUM], // size(0x100), align(0x1000)
    avail: VirtqAvail,                  // size(0x26)
    _pad: [u8; 0xeda],                   // size(0x1000 - (0x100 + 0x26) = 0xeda)
    used: VirtqUsed,                    // size(_), align(0x1000)
    queue_idx: u32,
    used_idx: *mut u16,
    last_used_idx: u16,
}


impl Virtq {
    unsafe fn init(index: u32) -> *mut Self {
        let virtq_paddr =
            alloc_pages(align_up(mem::size_of::<Virtq>() as usize, PAGE_SIZE) / PAGE_SIZE);
        let vq = virtq_paddr.unwrap().addr as *mut Virtq;
        let virtq = vq.as_mut().unwrap();
        (*vq).queue_idx = index;
        let used_idx = (&mut (virtq.used) as *const VirtqUsed as *const u8)
            .offset(mem::offset_of!(VirtqUsed, idx) as isize);
        virtq.used_idx = used_idx as *mut u16;

        // 1. Select the queue writing its index (first queue is 0) to QueueSel.
        virtio_reg_write32(VIRTIO_REG_QUEUE_SEL, index);
        // 5. Notify the device about the queue size by writing the size to QueueNum.
        virtio_reg_write32(VIRTIO_REG_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
        // 6. Notify the device about the used alignment by writing its value in bytes to QueueAlign.
        virtio_reg_write32(VIRTIO_REG_QUEUE_ALIGN, 0);
        // 7. Write the physical number of the first page of the queue to the QueuePFN register.
        virtio_reg_write32(VIRTIO_REG_QUEUE_PFN, virtq_paddr.unwrap().addr as u32);

        vq
    }

    fn kick(&mut self, desc_index: u16) {
        self.avail.ring[self.avail.idx as usize % VIRTQ_ENTRY_NUM] = desc_index;
        self.avail.idx += 1;
        compiler_fence(SeqCst);
        virtio_reg_write32(VIRTIO_REG_QUEUE_NOTIFY, self.queue_idx);
        self.last_used_idx += 1;
    }

    fn is_busy(&self) -> bool {
        unsafe {
            self.last_used_idx != *self.used_idx
        }
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct VirtioBlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
    data: [u8; 512],
    status: u8,
}

fn virtio_reg_read32(offset: usize) -> u32 {
    unsafe {
        ((VIRTIO_BLK_PADDR + offset) as *const u32).read_volatile()
    }
}

fn virtio_reg_read64(offset: usize) -> u64 {
    unsafe {
        ((VIRTIO_BLK_PADDR + offset) as *const u64).read_volatile()
    }
}

fn virtio_reg_write32(offset: usize, value: u32) {
    unsafe {
        ((VIRTIO_BLK_PADDR + offset) as *mut u32).write_volatile(value)
    }
}

fn virtio_reg_fetch_and_or32(offset: usize, value: u32) {
    virtio_reg_write32(offset, virtio_reg_read32(offset) | value);
}

static mut BLK_REQUEST_VQ: *mut Virtq = ptr::null_mut();
static mut BLK_REQ: *mut VirtioBlkReq = ptr::null_mut();
static mut BLK_REQ_PADDR: usize =  0;
static mut BLK_CAPACITY: u32 = 0;

pub unsafe fn init() {
    assert!(virtio_reg_read32(VIRTIO_REG_MAGIC) == 0x74726976);
    assert!(virtio_reg_read32(VIRTIO_REG_VERSION) == 1);
    assert!(virtio_reg_read32(VIRTIO_REG_DEVICE_ID) == VIRTIO_DEVICE_BLK);

    // 1. Reset the device.
    virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, 0);
    // 2. Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
    virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_ACK);
    // 3. Set the DRIVER status bit: the guest OS knows how to drive the device.
    virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER);
    // 5. Set the FEATURES_OK status bit.
    virtio_reg_fetch_and_or32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_FEAT_OK);
    // 7. Perform device-specific setup, including discovery of virtqueues for the device
    BLK_REQUEST_VQ = Virtq::init(0);
    // 8. Set the DRIVER_OK status bit.
    virtio_reg_write32(VIRTIO_REG_DEVICE_STATUS, VIRTIO_STATUS_DRIVER_OK);

    // ディスク容量を取得
    BLK_CAPACITY = virtio_reg_read64(VIRTIO_REG_DEVICE_CONFIG + 0) as u32 * SECTOR_SIZE;
    println!("virtio-blk: capacity is {BLK_CAPACITY} bytes");

    // デバイスへの処理要求を格納する領域を確保
    BLK_REQ_PADDR =
        alloc_pages(align_up(mem::size_of::<VirtioBlkReq>() as usize, PAGE_SIZE) / PAGE_SIZE).unwrap().addr;
    BLK_REQ = BLK_REQ_PADDR as *mut VirtioBlkReq;
}

pub unsafe fn read_write_disk(buf: *mut u8, sector: u32, is_write: bool) -> Result<(), ()> {
    if sector >= BLK_CAPACITY / SECTOR_SIZE {
        println!(
            "virtio: tried to read/write sector={sector}, but capacity is {}",
            BLK_CAPACITY / SECTOR_SIZE
        );
        return Err(());
    }

    // リクエストを構築する
    let blk_req = BLK_REQ.as_mut().unwrap();
    blk_req.sector = sector as u64;
    blk_req.type_ = if is_write {
        VIRTIO_BLK_T_OUT as u32
    } else {
        VIRTIO_BLK_T_IN as u32
    };
    if is_write {
        ptr::copy(
            buf,
            &mut blk_req.data as *mut [u8] as *mut u8,
            SECTOR_SIZE as usize,
        );
    }

    // virtqueueのディスクリプタを構築する
    let vq = BLK_REQUEST_VQ.as_mut().unwrap();
    vq.desc[0].addr = BLK_REQ_PADDR;
    vq.desc[0].len = (mem::size_of::<u32>() * 2 + mem::size_of::<u64>()) as u32;
    vq.desc[0].flags = VIRTQ_DESC_F_NEXT;
    vq.desc[0].next = 1;

    vq.desc[1].addr = BLK_REQ_PADDR + mem::offset_of!(VirtioBlkReq, data) as usize;
    vq.desc[1].len = SECTOR_SIZE;
    vq.desc[1].flags = VIRTQ_DESC_F_NEXT | if is_write { 0 } else { VIRTQ_DESC_F_WRITE };
    vq.desc[1].next = 2;

    vq.desc[2].addr = BLK_REQ_PADDR + mem::offset_of!(VirtioBlkReq, status) as usize;
    vq.desc[2].len = mem::size_of::<u8>() as u32;
    vq.desc[2].flags = VIRTQ_DESC_F_WRITE;

    // デバイスに新しいリクエストがあることを通知する
    Virtq::kick(vq, 0);

    while vq.is_busy() {}

    // 0でない値が帰ってきたらエラー
    if blk_req.status != 0 {
        println!(
            "virtio: warn: failed to read/write sector={sector}, status={}",
            blk_req.status
        );
        return Err(());
    }

    // 読み込み処理の場合は、バッファにデータをコピーする
    if !is_write {
        ptr::copy(
            &blk_req.data as *const [u8] as *const u8,
            buf,
            SECTOR_SIZE as usize,
        );
    }

    Ok(())
}
