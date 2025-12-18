use crate::{block::BlockDevice, boot_sector::BootSector, error::FatError, fat::Fat, directory::DirEntry};

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

    pub fn cluster_lba(&self, cluster: u32) -> u64 {
        let data_start = self.boot.reserved_sectors as u64
            + (self.boot.fat_count as u64 * self.boot.fat_size_sectors as u64);
        data_start + ((cluster - 2) as u64 * self.boot.sectors_per_cluster as u64)
    }

    /// Write clusters
    pub fn write_clusters(&mut self, chain: &[u32], data: &[u8]) {
        let mut offset = 0;
        let mut sector_buf = [0u8; 512];

        for &cluster in chain {
            let lba = self.cluster_lba(cluster);

            for s in 0..self.boot.sectors_per_cluster {
                sector_buf.fill(0);

                let end = core::cmp::min(offset + 512, data.len());
                if offset < data.len() {
                    sector_buf[..end - offset].copy_from_slice(&data[offset..end]);
                }

                self.device.write_sector(lba + s as u64, &sector_buf);
                offset += 512;
            }
        }
    }

    /// Create file
    pub fn create_file(
        &mut self,
        name: [u8; 11],
        data: &[u8],
    ) -> Result<(), FatError> {
        let cluster_size = self.cluster_size() as usize;
        let cluster_needed = (data.len() + cluster_size - 1) / cluster_size;

        let fat_start_lba = self.boot.reserved_sectors as u64;
        let mut fat = Fat::load(
            &mut self.device,
            fat_start_lba,
            self.boot.fat_size_sectors,
        );

        let cluster_chain = fat.allocate_chain(cluster_needed);
        self.write_clusters(&cluster_chain, data);
        fat.flush(&mut self.device, fat_start_lba);

        let entry = DirEntry::new(
            name,
            cluster_chain[0],
            data.len() as u32,
        );

        self.write_root_entry(&entry)?;

        Ok(())
    }

    /// List root directory entries
    pub fn list_root(&self) -> Result<alloc::vec::Vec<DirEntry>, crate::error::FatError> {
        let mut sector_buf = [0u8; 512];
        let lba = self.cluster_lba(self.root_cluster());

        self.device.read_sector(lba, &mut sector_buf);

        let mut files = alloc::vec::Vec::new();

        for i in 0..16 {
            let offset = i * 32;
            let entry = &sector_buf[offset..offset + 32];
            if entry[0] == 0x00 || entry[0] == 0xE5 {
                continue; // Unused or deleted entry
            }
            let mut name = [0u8; 11];
            name.copy_from_slice(&entry[0..11]);

            let first_cluster = u16::from_le_bytes([entry[20], entry[21]]) as u32
                | ((u16::from_le_bytes([entry[12], entry[13]]) as u32) << 16);

            let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);

            files.push(DirEntry {
                name,
                attributes: entry[11],
                first_cluster,
                size,
            });
        }

        Ok(files)
    }


    /// Write root directory entry
    fn write_root_entry(&mut self, entry: &DirEntry) -> Result<(), FatError> {
        let root_lba = self.cluster_lba(self.root_cluster());
        let mut sector_buf = [0u8; 512];

        self.device.read_sector(root_lba, &mut sector_buf);

        for i in 0..16 {
            let offset = i * 32;
            if sector_buf[offset] == 0x00 || sector_buf[offset] == 0xE5 {
                sector_buf[offset..offset + 32].copy_from_slice(&entry.serialize());
                self.device.write_sector(root_lba, &sector_buf);
                return Ok(());
            }
        }

        Err(FatError::NoFreeDirectoryEntry)
    }
}