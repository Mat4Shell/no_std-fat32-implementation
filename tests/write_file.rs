use no_std::volume::Fat32Volume;

mod block_device;
mod image;

use block_device::MemBlockDevice;
use image::create_test_image;

#[test]
fn create_file_is_persistent() {
    let img = create_test_image();
    let mut device = MemBlockDevice { data: img };

    {
        // Open FS and create file
        let mut fs = Fat32Volume::open(&mut device).unwrap();

        fs.create_file(
            *b"HELLO   TXT",
            b"Hello FAT32!",
        ).unwrap();
    }

    {
        // Re-open FS and read directory entry
        let fs = Fat32Volume::open(&mut device).unwrap();
        let files = fs.list_root().unwrap();

        let file = files.iter()
            .find(|f| &f.name == b"HELLO   TXT")
            .expect("File not found");

        assert_eq!(file.size, 12);
    }
}
