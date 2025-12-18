use no_std::block::BlockDevice;

/// BlockDevice backed by a memory buffer.
/// Simulates a real disk image.
pub struct MemBlockDevice {
    pub data: Vec<u8>,
}

impl BlockDevice for &mut MemBlockDevice {
    fn read_sector(&self, lba: u64, buf: &mut [u8]) {
        let start = lba as usize * 512;
        buf.copy_from_slice(&self.data[start..start + 512]);
    }

    fn write_sector(&mut self, lba: u64, buf: &[u8]) {
        let start = lba as usize * 512;
        self.data[start..start + 512].copy_from_slice(buf);
    }
}
