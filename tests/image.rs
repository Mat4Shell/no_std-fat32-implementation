pub fn create_test_image() -> Vec<u8> {
    let sectors = 100;
    let mut img = vec![0u8; sectors * 512];

    // Boot sector
    img[11..13].copy_from_slice(&512u16.to_le_bytes()); // bytes/sector
    img[13] = 1; // sectors/cluster
    img[14..16].copy_from_slice(&1u16.to_le_bytes()); // reserved
    img[16] = 1; // FAT count
    img[32..36].copy_from_slice(&(sectors as u32).to_le_bytes()); // total sectors
    img[36..40].copy_from_slice(&1u32.to_le_bytes()); // FAT size
    img[44..48].copy_from_slice(&2u32.to_le_bytes()); // root cluster

    // FAT (sector 1)
    let fat_offset = 512;
    img[fat_offset + 0..fat_offset + 4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes());
    img[fat_offset + 4..fat_offset + 8].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
    img[fat_offset + 8..fat_offset + 12].copy_from_slice(&0u32.to_le_bytes()); // cluster 2 free

    img
}
