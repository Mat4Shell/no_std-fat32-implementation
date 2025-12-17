#[derive(Debug)]
pub enum FatError {
    InvalidBootSector,
    IoError,
    NotFound,
}
