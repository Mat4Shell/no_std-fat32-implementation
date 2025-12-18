use std::fs::{File, create_dir_all};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

const IMG_SIZE_MB: u64 = 64;
const SECTOR_SIZE: u16 = 512;
const SECTORS_PER_CLUSTER: u8 = 8;
const RESERVED_SECTORS: u16 = 32;
const FAT_COUNT: u8 = 2;
const FAT_SIZE_SECTORS: u32 = 128;
const ROOT_CLUSTER: u32 = 2;

#[test]
fn create_fat32_image() {
    let out_dir = Path::new("test_images");
    create_dir_all(out_dir).unwrap();

    let img_path = out_dir.join("fat32.img");
    let mut file = File::create(&img_path).unwrap();

    // Set image size
    file.seek(SeekFrom::Start(IMG_SIZE_MB * 1024 * 1024 - 1))
        .unwrap();
    file.write_all(&[0]).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();

    let total_sectors = (IMG_SIZE_MB * 1024 * 1024) / SECTOR_SIZE as u64;

    // ------------------
    // BOOT SECTOR
    // ------------------
    let mut boot = [0u8; 512];
    boot[0..3].copy_from_slice(&[0xEB, 0x58, 0x90]); // jump instruction
    boot[3..11].copy_from_slice(b"MSWIN4.1");       // OEM Name

    boot[11..13].copy_from_slice(&SECTOR_SIZE.to_le_bytes()); // bytes per sector
    boot[13] = SECTORS_PER_CLUSTER;                             // sectors per cluster
    boot[14..16].copy_from_slice(&RESERVED_SECTORS.to_le_bytes()); // reserved sectors
    boot[16] = FAT_COUNT;                                        // number of FATs
    boot[17..19].copy_from_slice(&0u16.to_le_bytes());           // root entries (0 for FAT32)
    boot[19..21].copy_from_slice(&0u16.to_le_bytes());           // total sectors (16-bit, 0 for FAT32)
    boot[21] = 0xF8;                                             // media descriptor
    boot[22..24].copy_from_slice(&0u16.to_le_bytes());           // fat size 16-bit (0 for FAT32)
    boot[24..26].copy_from_slice(&63u16.to_le_bytes());          // sectors per track
    boot[26..28].copy_from_slice(&255u16.to_le_bytes());         // number of heads
    boot[28..32].copy_from_slice(&0u32.to_le_bytes());           // hidden sectors
    boot[32..36].copy_from_slice(&(total_sectors as u32).to_le_bytes()); // total sectors 32-bit
    boot[36..40].copy_from_slice(&FAT_SIZE_SECTORS.to_le_bytes()); // FAT size 32-bit

    boot[40..42].copy_from_slice(&0u16.to_le_bytes()); // flags
    boot[42..44].copy_from_slice(&0u16.to_le_bytes()); // version
    boot[44..48].copy_from_slice(&ROOT_CLUSTER.to_le_bytes()); // root cluster
    boot[48..50].copy_from_slice(&1u16.to_le_bytes()); // FSInfo sector
    boot[50..52].copy_from_slice(&6u16.to_le_bytes()); // backup boot sector

    boot[64] = 0x80;                        // drive number
    boot[66] = 0x29;                        // boot signature
    boot[67..71].copy_from_slice(&0x12345678u32.to_le_bytes()); // volume ID
    boot[71..82].copy_from_slice(b"NO_STD_FAT ");               // volume label
    boot[82..90].copy_from_slice(b"FAT32   ");                  // filesystem type

    boot[510] = 0x55;
    boot[511] = 0xAA;

    file.write_all(&boot).unwrap();

    // ------------------
    // FSInfo sector
    // ------------------
    let mut fsinfo = [0u8; 512];
    fsinfo[0..4].copy_from_slice(&0x41615252u32.to_le_bytes());   // lead signature
    fsinfo[484..488].copy_from_slice(&0x61417272u32.to_le_bytes()); // struct signature
    fsinfo[488..492].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // free cluster count (unknown)
    fsinfo[492..496].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // next free cluster (unknown)
    fsinfo[510] = 0x55;
    fsinfo[511] = 0xAA;

    file.seek(SeekFrom::Start(512)).unwrap();
    file.write_all(&fsinfo).unwrap();

    // ------------------
    // Backup Boot Sector
    // ------------------
    file.seek(SeekFrom::Start(6 * 512)).unwrap();
    file.write_all(&boot).unwrap();

    // ------------------
    // FAT Tables
    // ------------------
    for fat_index in 0..FAT_COUNT {
        let fat_start = (RESERVED_SECTORS as u64
            + fat_index as u64 * FAT_SIZE_SECTORS as u64)
            * SECTOR_SIZE as u64;

        file.seek(SeekFrom::Start(fat_start)).unwrap();

        let mut fat = vec![0u8; FAT_SIZE_SECTORS as usize * 512];

        // FAT[0] & FAT[1] reserved
        fat[0..4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes());
        fat[4..8].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());

        // Root directory cluster
        fat[8..12].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes());

        file.write_all(&fat).unwrap();
    }

    println!(
        "FAT32 Microsoft-compliant image created at {:?} ({} MB)",
        img_path, IMG_SIZE_MB
    );
}
