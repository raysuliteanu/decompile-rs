use std::io::Read;

use log::{debug, trace};

use crate::types::{
    AttributeInfo, ClassFile, ConstantPoolType, CpInfo, FieldAccessFlags, FieldInfo,
    MethodAccessFlags, MethodInfo,
};

const CAFE_BABE: u32 = 0xCAFE_BABE;

pub struct Decompile<'a> {
    file: &'a mut std::fs::File,
}

impl<'a> Decompile<'a> {
    pub fn new(file: &'a mut std::fs::File) -> Self {
        Self { file }
    }

    pub fn decompile(&mut self) -> std::io::Result<()> {
        let magic = self.read_u32();
        if magic != CAFE_BABE {
            // todo: return Err
            panic!("invalid magic number");
        }

        let mut class_file = ClassFile::default();
        class_file.magic = magic;
        class_file.minor_version = self.read_u16();
        class_file.major_version = self.read_u16();

        debug!(
            "Class Version: {}.{}",
            class_file.major_version, class_file.minor_version
        );

        // constant pool is indexed starting at 1 so put in a dummy at index 0
        // See https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.1
        // constant_pool[]
        //      "The constant_pool table is indexed from 1 to constant_pool_count - 1."
        class_file.constant_pool.push(CpInfo { tag: 0, info: None });

        class_file.constant_pool_count = self.read_u16();

        debug!("constant pool count: {}", class_file.constant_pool_count);

        // -1 because see comment above
        for _ in 0..class_file.constant_pool_count - 1 {
            let cp_info_tag = self.read_u8();
            let cp_info_type = match cp_info_tag {
                1 => {
                    // utf8
                    let len = self.read_u16();
                    let bytes = self.read_variable(len as usize);
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
                7 => {
                    // class
                    ConstantPoolType::ConstantClass {
                        name_idx: self.read_u16(),
                    }
                }
                8 => {
                    // string
                    ConstantPoolType::ConstantString {
                        string_idx: self.read_u16(),
                    }
                }
                9 => {
                    // fieldref
                    ConstantPoolType::ConstantFieldref {
                        class_index: self.read_u16(),
                        name_and_type_idx: self.read_u16(),
                    }
                }
                10 => {
                    // methodref
                    ConstantPoolType::ConstantMethodref {
                        class_index: self.read_u16(),
                        name_and_type_idx: self.read_u16(),
                    }
                }
                11 => {
                    // interfacemethodref
                    ConstantPoolType::ConstantInterfaceMethodref {
                        class_index: self.read_u16(),
                        name_and_type_idx: self.read_u16(),
                    }
                }
                12 => {
                    // nameandtype
                    ConstantPoolType::ConstantNameAndType {
                        name_idx: self.read_u16(),
                        desc_idx: self.read_u16(),
                    }
                }
                15 => {
                    // methodhandle
                    ConstantPoolType::ConstantMethodHandle {
                        ref_kind: self.read_u8(),
                        ref_idx: self.read_u16(),
                    }
                }
                16 => {
                    // methodtype
                    ConstantPoolType::ConstantMethodType {
                        desc_idx: self.read_u16(),
                    }
                }
                17 => {
                    // dynamic
                    ConstantPoolType::ConstantDynamic {
                        bootstrap_method_attr_index: self.read_u16(),
                        name_and_type_index: self.read_u16(),
                    }
                }
                18 => {
                    // invokedynamic
                    ConstantPoolType::ConstantInvokeDynamic {
                        bootstrap_method_attr_index: self.read_u16(),
                        name_and_type_index: self.read_u16(),
                    }
                }
                19 => {
                    // module
                    ConstantPoolType::ConstantModule {
                        name_idx: self.read_u16(),
                    }
                }
                20 => {
                    // package
                    ConstantPoolType::ConstantPackage {
                        name_idx: self.read_u16(),
                    }
                }
                _ => panic!("invalid cp_info tag: {cp_info_tag}"),
            };

            let info = CpInfo {
                tag: cp_info_tag,
                info: Some(cp_info_type),
            };

            debug!("adding {:?}", info);

            class_file.constant_pool.push(info);
        }

        debug!(
            "read {} constant pool items",
            class_file.constant_pool.len()
        );

        class_file.access_flags = self.read_u16();

        class_file.this_class = self.read_u16();

        class_file.super_class = self.read_u16();

        class_file.interfaces_count = self.read_u16();

        for _ in 0..class_file.interfaces_count {
            class_file.interfaces.push(self.read_u8());
        }

        class_file.fields_count = self.read_u16();

        for _ in 0..class_file.fields_count {
            class_file.fields.push(self.read_field_info());
        }

        class_file.methods_count = self.read_u16();

        for _ in 0..class_file.methods_count {
            class_file.methods.push(self.read_method_info());
        }

        trace!("class file: {:?}", class_file);

        // TODO: validate class file e.g. indexes into constant pool are valid

        // TODO: print disassembly

        Ok(())
    }

    fn read_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        self.file.read_exact(&mut buf).expect("invalid class file"); // todo: better error
        u32::from_be_bytes(buf)
    }

    fn read_u16(&mut self) -> u16 {
        let mut buf = [0u8; 2];
        self.file.read_exact(&mut buf).expect("invalid class file"); // todo: better error
        u16::from_be_bytes(buf)
    }

    fn read_u8(&mut self) -> u8 {
        let mut buf = [0u8; 1];
        self.file.read_exact(&mut buf).expect("invalid class file"); // todo: better error
        u8::from_be_bytes(buf)
    }

    fn read_variable(&mut self, len: usize) -> Vec<u8> {
        let mut buf = vec![0; len];
        self.file.read_exact(&mut buf).expect("invalid class file"); // todo: better error
        buf
    }

    fn read_field_info(&mut self) -> FieldInfo {
        let access_flags = self.read_u16();
        let name_index = self.read_u16();
        let descriptor_index = self.read_u16();
        let attributes_count = self.read_u16();

        let mut field_info = FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attributes: vec![],
        };

        for _ in 0..attributes_count {
            field_info.attributes.push(self.read_attribute_info());
        }

        field_info
    }

    fn read_method_info(&mut self) -> MethodInfo {
        let access_flags = self.read_u16();
        let name_index = self.read_u16();
        let descriptor_index = self.read_u16();
        let attributes_count = self.read_u16();

        let mut method_info = MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attributes: vec![],
        };

        for _ in 0..attributes_count {
            method_info.attributes.push(self.read_attribute_info());
        }

        method_info
    }

    fn read_attribute_info(&mut self) -> AttributeInfo {
        let index = self.read_u16();
        let length = self.read_u32();

        AttributeInfo {
            attribute_name_index: index,
            attribute_length: length,
            info: self.read_variable(length as usize),
        }
    }
}
