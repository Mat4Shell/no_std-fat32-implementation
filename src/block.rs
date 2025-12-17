/// A block device able to read/write 512-byte sectors.
///
/// This abstraction allows the FAT32 parser to work in no_std
/// environments (bootloader, firmware, embedded, etc.).
pub trait BlockDevice {
    /// Read a sector at the given LBA into `buf`.
    fn read_sector(&self, lba: u64, buf: &mut [u8]);

    /// Write a sector at the given LBA from `buf`.
    fn write_sector(&mut self, lba: u64, buf: &[u8]);
}
