use libzoea::caps::CNode;
use libzoea::caps::PageTable;
use libzoea::caps::PageTableCapability;
use libzoea::shared::types::BootInfo;
use libzoea::caps::UntypedCapability;
use libzoea::caps::CNodeCapability;


pub const ROOT_CNODE_RADIX: u32 = 18;

pub fn get_untyped(boot_info: &BootInfo) -> UntypedCapability {
    UntypedCapability::from_untyped_info(ROOT_CNODE_RADIX, &boot_info.untyped_infos[0])
}

pub fn get_root_cnode(boot_info: &BootInfo) -> CNodeCapability {
    CNodeCapability {
        cap_ptr: boot_info.root_cnode_idx,
        cap_depth: ROOT_CNODE_RADIX,
        cap_data: CNode {
            radix: ROOT_CNODE_RADIX,
            cursor: boot_info.firtst_empty_idx,
        }
    }
}

pub fn get_root_vspace(boot_info: &BootInfo) -> PageTableCapability {
    PageTableCapability {
        cap_ptr: boot_info.root_vspace_idx,
        cap_depth: ROOT_CNODE_RADIX,
        cap_data: PageTable {
            mapped_address: 0, // nonsense
            is_root: true,
            is_mapped: true
        }
    }
}
