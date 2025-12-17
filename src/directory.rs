use alloc::vec::Vec;

/// FAT32 short directory entry
#[derive(Debug)]
pub struct DirEntry {
    pub name: [u8; 11],
    pub attributes: u8,
    pub first_cluster: u32,
    pub size: u32,
}
