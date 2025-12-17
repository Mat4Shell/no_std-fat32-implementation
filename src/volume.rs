use crate::{block::BlockDevice, boot_sector::BootSector, error::FatError};

/// FAT32 volume representation
pub struct Fat32Volume<B: BlockDevice> {
    pub boot: BootSector,
    device: B,
}

impl<B: BlockDevice> Fat32Volume<B> {
    /// Open a FAT32 volume from a block device.
    pub fn open(device: B) -> Result<Self, FatError> {
        let mut sector = [0u8; 512];
        device.read_sector(0, &mut sector);

        let boot = BootSector::parse(&sector)?;
        Ok(Self { boot, device })
    }

    /// Volume size in bytes.
    pub fn volume_size(&self) -> u64 {
        self.boot.total_sectors as u64 * self.boot.bytes_per_sector as u64
    }

    /// Cluster size in bytes.
    pub fn cluster_size(&self) -> u32 {
        self.boot.cluster_size()
    }

    /// Number of FAT tables.
    pub fn fat_count(&self) -> u8 {
        self.boot.fat_count
    }

    /// Root directory first cluster.
    pub fn root_cluster(&self) -> u32 {
        self.boot.root_cluster
    }
}
