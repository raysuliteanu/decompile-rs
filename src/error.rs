use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DecompileError {
    #[error("failed to parse input file: {0}\t see prior errors")]
    InvalidInputFile(PathBuf),
    #[error("invalid magic number: 0x{0:X}")]
    InvalidMagicNumber(u32),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("no such file: {0}")]
    NoSuchFile(PathBuf),
}
