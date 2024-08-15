use std::io::Read;

use log::debug;

use crate::types::{ClassFile, ConstantPoolType, CpInfo};

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

        self.file.read(&mut buf)?;
        class_file.major_version = u16::from_be_bytes(buf);

        debug!(
            "Class Version: {}.{}",
            class_file.major_version, class_file.minor_version
        );

        self.file.read(&mut buf)?;
        class_file.constant_pool_count = u16::from_be_bytes(buf);

        debug!("constant pool count: {}", class_file.constant_pool_count);
        for _ in [0..class_file.constant_pool_count] {
            let mut buf = [0u8; 1];
            self.file.read_exact(&mut buf)?;
            let cp_info_tag = u8::from_be_bytes(buf);
            let cp_info_type = match cp_info_tag {
                1 => {
                    // utf8
                    let mut buf = [0u8; 2];
                    self.file.read_exact(&mut buf)?;
                    let len = u16::from_be_bytes(buf);
                    let mut bytes = Vec::with_capacity(len as usize);
                    self.file.read_exact(&mut bytes)?;
                    let value = std::str::from_utf8(&bytes).unwrap().to_string();

                    ConstantPoolType::ConstantUtf8 { len, value }
                }
                3 => {
                    // integer
                    let mut buf = [0u8; 4];
                    self.file.read_exact(&mut buf)?;
                    let value = i32::from_be_bytes(buf);

                    ConstantPoolType::ConstantInteger { value }
                }
                4 => {
                    // float
                    let mut buf = [0u8; 4];
                    self.file.read_exact(&mut buf)?;
                    let value = f32::from_be_bytes(buf);

                    ConstantPoolType::ConstantFloat { value }
                }
                5 => {
                    // long
                    let mut buf = [0u8; 8];
                    self.file.read_exact(&mut buf)?;
                    let value = i64::from_be_bytes(buf);

                    ConstantPoolType::ConstantLong { value }
                }
                6 => {
                    // double
                    let mut buf = [0u8; 8];
                    self.file.read_exact(&mut buf)?;
                    let value = f64::from_be_bytes(buf);

                    ConstantPoolType::ConstantDouble { value }
                }
                /*
                                // class
                                7 => {},
                                // string
                                8 => {},
                                // fieldref
                                9 => {},
                                // methodref
                                10 => {},
                                // interfacemethodref
                                11 => {},
                                // nameandtype
                                12 => {},
                                // methodhandle
                                15 => {},
                                // methodtype
                                16 => {},
                                // dynamic
                                17 => {},
                                // invokedynamic
                                18 => {},
                                // module
                                19 => {},
                                // package
                                20 => {},
                */
                _ => panic!("invalid cp_info tag: {cp_info_tag}"),
            };

            class_file.constant_pool.push(CpInfo {
                tag: cp_info_tag,
                info: Some(cp_info_type),
            });
        }

        Ok(())
    }
}
