use no_std::volume::Fat32Volume;
use no_std::block::BlockDevice;

struct MemBlockDevice {
    data: [u8; 512],
}

impl BlockDevice for MemBlockDevice {
    fn read_sector(&self, _lba: u64, buf: &mut [u8]) {
        buf.copy_from_slice(&self.data);
    }
    fn write_sector(&mut self, _lba: u64, _buf: &[u8]) {}
}

#[test]
fn parse_boot_sector() {
    let mut img = [0u8; 512];
    img[11..13].copy_from_slice(&512u16.to_le_bytes());
    img[13] = 8;
    img[14..16].copy_from_slice(&32u16.to_le_bytes());
    img[16] = 2;
    img[36..40].copy_from_slice(&1000u32.to_le_bytes());
    img[44..48].copy_from_slice(&2u32.to_le_bytes());

    let dev = MemBlockDevice { data: img };
    let vol = Fat32Volume::open(dev).unwrap();

    assert_eq!(vol.cluster_size(), 4096);
    assert_eq!(vol.fat_count(), 2);
    assert_eq!(vol.root_cluster(), 2);
}
