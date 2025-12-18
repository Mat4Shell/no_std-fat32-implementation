use alloc::vec::Vec;

/// FAT32 short directory entry
#[derive(Debug)]
pub struct DirEntry {
    pub name: [u8; 11],
    pub attributes: u8,
    pub first_cluster: u32,
    pub size: u32,
}

impl DirEntry {
    /// Create a new directory entry
    pub fn new(name: [u8; 11], first_cluster: u32, size: u32) -> Self {
        Self {
            name,
            attributes: 0x20,
            first_cluster,
            size,
        }
    }

    pub fn serialize(&self) -> [u8; 32] {
        let mut entry = [0u8; 32];
        entry[0..11].copy_from_slice(&self.name);
        entry[11] = self.attributes;
        entry[20..22].copy_from_slice(&(self.first_cluster as u16).to_le_bytes());
        entry[26..28].copy_from_slice(&((self.first_cluster & 0xFFFF) as u16).to_le_bytes());
        entry[28..32].copy_from_slice(&self.size.to_le_bytes());
        entry
    }
}