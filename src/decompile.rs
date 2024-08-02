use std::io::Read;

use crate::types::ClassFile;

const CAFE_BABE: u32 = 0xCAFE_BABE;

pub struct Decompile<'a> {
    file: &'a mut std::fs::File,
}

impl<'a> Decompile<'a> {
    pub fn new(file: &'a mut std::fs::File) -> Self {
        Self { file }
    }

    pub fn decompile(&mut self) -> std::io::Result<()> {
        let mut buf = [0u8; 4];
        self.file.read(&mut buf)?;
        let magic = u32::from_be_bytes(buf);
        if magic != CAFE_BABE {
            // todo: return Err
            panic!("invalid magic number");
        }

        let mut class_file = ClassFile::default();
        class_file.magic = magic;

        let mut buf = [0u8; 2];
        self.file.read(&mut buf)?;
        class_file.minor_version = u16::from_be_bytes(buf);

        let mut buf = [0u8; 2];
        self.file.read(&mut buf)?;
        class_file.major_version = u16::from_be_bytes(buf);

        println!(
            "Class Version: {}.{}",
            class_file.major_version, class_file.minor_version
        );

        //        let mut buf = Vec::new();
        //       let _read = self.file.read_to_end(&mut buf);

        Ok(())
    }
}
