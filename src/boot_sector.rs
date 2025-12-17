use crate::error::FatError;

/// FAT32 BIOS Parameter Block (partial)
#[derive(Debug, Clone)]
pub struct BootSector {
    /// Bytes per sector (usually 512)
    pub bytes_per_sector: u16,
    /// Sectors per cluster
    pub sectors_per_cluster: u8,
    /// Reserved sectors before the FAT
    pub reserved_sectors: u16,
    /// Number of FATs
    pub fat_count: u8,
    /// Total sectors of the volume
    pub total_sectors: u32,
    /// Size of one FAT in sectors
    pub fat_size_sectors: u32,
    /// Root directory first cluster
    pub root_cluster: u32,
}

impl BootSector {
    /// Parse a FAT32 boot sector from raw sector data.
    pub fn parse(sector: &[u8]) -> Result<Self, FatError> {
        if sector.len() < 512 {
            return Err(FatError::InvalidBootSector);
        }

        let bytes_per_sector = u16::from_le_bytes([sector[11], sector[12]]);
        let sectors_per_cluster = sector[13];
        let reserved_sectors = u16::from_le_bytes([sector[14], sector[15]]);
        let fat_count = sector[16];

        let total_sectors_16 = u16::from_le_bytes([sector[19], sector[20]]);
        let total_sectors_32 =
            u32::from_le_bytes([sector[32], sector[33], sector[34], sector[35]]);

        let total_sectors = if total_sectors_16 != 0 {
            total_sectors_16 as u32
        } else {
            total_sectors_32
        };

        let fat_size_sectors =
            u32::from_le_bytes([sector[36], sector[37], sector[38], sector[39]]);

        let root_cluster =
            u32::from_le_bytes([sector[44], sector[45], sector[46], sector[47]]);

        Ok(Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            fat_count,
            total_sectors,
            fat_size_sectors,
            root_cluster,
        })
    }

    /// Cluster size in bytes.
    pub fn cluster_size(&self) -> u32 {
        self.bytes_per_sector as u32 * self.sectors_per_cluster as u32
    }
}
