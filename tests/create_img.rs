#[test]
fn create_fat32_image() {
    use std::fs::File;
    use std::io::{Write, Seek, SeekFrom};

    fn create_empty_img(path: &str, size_mb: usize) {
        let mut file = File::create(path).unwrap();
        file.seek(SeekFrom::Start((size_mb * 1024 * 1024 - 1) as u64)).unwrap();
        file.write_all(&[0]).unwrap();
    }

    create_empty_img("test_images/fat32.img", 10); // crée 10 Mo
    println!("Image fat32.img créée !");
}
