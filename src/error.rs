use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DecompileError {
    #[error("invalid magic number: 0x{0:X}")]
    InvalidMagicNumber(u32),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("no such file: {0}")]
    NoSuchFile(PathBuf),
    #[error("invalid cp_info tag '{0}' at offset {1}")]
    InvalidConstantPoolTag(u8, u64),
    #[error("no such constant pool index '{0}' at offset {1}")]
    NoSuchConstantPoolEntry(u16, u64),
    #[error("invalid Constant_UTF8 at '{0}'")]
    InvalidUtf8ConstantPoolEntry(u16),
}
