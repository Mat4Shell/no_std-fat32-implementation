use no_std::{volume::Fat32Volume, block::BlockDevice}; // ta lib

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

/// Simple BlockDevice basé sur un fichier, uniquement pour tests
pub struct FileBlockDevice {
    file: std::fs::File,
}

impl FileBlockDevice {
    pub fn open(path: &str) -> Self {
        let file = OpenOptions::new().read(true).write(true).open(path).unwrap();
        Self { file }
    }
}

impl BlockDevice for FileBlockDevice {
    fn read_sector(&self, lba: u64, buf: &mut [u8]) {
        let mut file = &self.file;
        file.seek(SeekFrom::Start(lba * 512)).unwrap();
        file.read_exact(buf).unwrap();
    }

    fn write_sector(&mut self, lba: u64, buf: &[u8]) {
        self.file.seek(SeekFrom::Start(lba * 512)).unwrap();
        self.file.write_all(buf).unwrap();
    }
}

#[test]
fn read_real_image() {
    let dev = FileBlockDevice::open(r"test_images/fat32.img");
    let vol = Fat32Volume::open(dev).unwrap();

    println!("Volume size: {} bytes", vol.volume_size());
    println!("Cluster size: {} bytes", vol.cluster_size());
    println!("Number of FATs: {}", vol.fat_count());
    println!("Root cluster: {}", vol.root_cluster());
}
