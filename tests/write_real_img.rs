use no_std::block::BlockDevice;
use no_std::volume::Fat32Volume;
use no_std::write::create_file;

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

/// BlockDevice basé sur un fichier réel (.img)
struct FileBlockDevice {
    file: std::fs::File,
}

impl FileBlockDevice {
    fn open(path: &str) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .expect("Cannot open image file");
        Self { file }
    }
}

impl BlockDevice for FileBlockDevice {
    fn read_sector(&self, lba: u64, buf: &mut [u8]) {
        let mut f = &self.file;
        f.seek(SeekFrom::Start(lba * 512)).unwrap();
        f.read_exact(buf).unwrap();
    }

    fn write_sector(&mut self, lba: u64, buf: &[u8]) {
        self.file.seek(SeekFrom::Start(lba * 512)).unwrap();
        self.file.write_all(buf).unwrap();
    }
}

#[test]
fn write_file_to_real_image() {
    let img_path = r"test_images/fat32.img";

    let dev = FileBlockDevice::open(img_path);
    let mut volume = Fat32Volume::open(dev).expect("Failed to open FAT32 volume");
    let root = volume.root_cluster();

    let content = b"Hello from no_std FAT32!\r\n";

    create_file(
        &mut volume,
        root,
        "HELLO.TXT",
        content,
    )
        .expect("Failed to create file");

    println!("File written successfully!");
}
