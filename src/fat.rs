use alloc::vec::Vec;
use crate::{block::BlockDevice, error::FatError};

/// FAT table handler
pub struct Fat {
    entries: Vec<u32>,
}

impl Fat {
    /// Load the FAT from disk.
    pub fn load<B: BlockDevice>(
        device: &mut B,
        fat_start_lba: u64,
        fat_size_sectors: u32,
    ) -> Self {
        let mut entries = Vec::new();
        let mut sector = [0u8; 512];

        for i in 0..fat_size_sectors {
            device.read_sector(fat_start_lba + i as u64, &mut sector);
            for chunk in sector.chunks_exact(4) {
                let val = u32::from_le_bytes(chunk.try_into().unwrap()) & 0x0FFFFFFF;
                entries.push(val);
            }
        }

        Self { entries }
    }

    /// Find a free cluster.
    pub fn find_free(&self) -> Option<u32> {
        self.entries
            .iter()
            .enumerate()
            .skip(2)
            .find(|(_, &v)| v == 0)
            .map(|(i, _)| i as u32)
    }

    /// Allocate a chain of clusters.
    pub fn allocate_chain(&mut self, count: usize) -> Vec<u32> {
        let mut chain = Vec::new();

        for _ in 0..count {
            let c = self.find_free().expect("No free clusters");
            self.entries[c as usize] = 0x0FFFFFFF;
            if let Some(&prev) = chain.last() {
                self.entries[prev as usize] = c;
            }
            chain.push(c);
        }

        chain
    }

    /// Write FAT back to disk.
    pub fn flush<B: BlockDevice>(
        &self,
        device: &mut B,
        fat_start_lba: u64,
    ) {
        let mut sector = [0u8; 512];
        let mut idx = 0;

        for lba in 0.. {
            if idx >= self.entries.len() {
                break;
            }

            for chunk in sector.chunks_exact_mut(4) {
                if idx < self.entries.len() {
                    chunk.copy_from_slice(&self.entries[idx].to_le_bytes());
                    idx += 1;
                }
            }

            device.write_sector(fat_start_lba + lba, &sector);
        }
    }
}
