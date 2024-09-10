use crate::error::DecompileError;
use crate::error::DecompileError::{InvalidMagicNumber, NoSuchFile};
use crate::types::{
    AttributeInfo, ClassFile, ConstantPoolType, CpInfo, FieldAccessFlags, FieldInfo,
    MethodAccessFlags, MethodInfo,
};
use log::{debug, trace};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

const CAFE_BABE: u32 = 0xCAFE_BABE;

pub type DecompileResult<T> = Result<T, DecompileError>;

pub struct Decompile {
    path: PathBuf,
}

impl Decompile {
    pub fn new(path: PathBuf) -> DecompileResult<Self> {
        if !path.exists() {
            return Err(NoSuchFile(path.clone()));
        }

        Ok(Self { path })
    }

    pub fn decompile(&mut self) -> DecompileResult<()> {
        let file = File::open(&self.path).map_err(DecompileError::IOError)?;

        let mut reader = BufReader::new(file);

        let magic = read_u32(&mut reader);
        if magic != CAFE_BABE {
            return Err(InvalidMagicNumber(magic));
        }

        let mut class_file = ClassFile::new(magic);
        class_file.minor_version = read_u16(&mut reader);
        class_file.major_version = read_u16(&mut reader);

        debug!(
            "Class Version: {}.{}",
            class_file.major_version, class_file.minor_version
        );

        // constant pool is indexed starting at 1 so put in a dummy at index 0
        // See https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.1
        // constant_pool[]
        //      "The constant_pool table is indexed from 1 to constant_pool_count - 1."
        class_file.constant_pool.push(CpInfo { tag: 0, info: None });

        class_file.constant_pool_count = read_u16(&mut reader);

        debug!("constant pool count: {}", class_file.constant_pool_count);

        // -1 because see comment above
        for _ in 0..class_file.constant_pool_count - 1 {
            let cp_info_tag = read_u8(&mut reader);
            let cp_info_type = match cp_info_tag {
                1 => utf8(&mut reader)?,
                3 => integer(&mut reader)?,
                4 => float(&mut reader)?,
                5 => long(&mut reader)?,
                6 => double(&mut reader)?,
                7 => ConstantPoolType::ConstantClass {
                    name_idx: read_u16(&mut reader),
                },
                8 => ConstantPoolType::ConstantString {
                    string_idx: read_u16(&mut reader),
                },
                9 => ConstantPoolType::ConstantFieldRef {
                    class_index: read_u16(&mut reader),
                    name_and_type_idx: read_u16(&mut reader),
                },
                10 => ConstantPoolType::ConstantMethodRef {
                    class_index: read_u16(&mut reader),
                    name_and_type_idx: read_u16(&mut reader),
                },
                11 => ConstantPoolType::ConstantInterfaceMethodRef {
                    class_index: read_u16(&mut reader),
                    name_and_type_idx: read_u16(&mut reader),
                },
                12 => ConstantPoolType::ConstantNameAndType {
                    name_idx: read_u16(&mut reader),
                    desc_idx: read_u16(&mut reader),
                },
                15 => ConstantPoolType::ConstantMethodHandle {
                    ref_kind: read_u8(&mut reader),
                    ref_idx: read_u16(&mut reader),
                },
                16 => ConstantPoolType::ConstantMethodType {
                    desc_idx: read_u16(&mut reader),
                },
                17 => ConstantPoolType::ConstantDynamic {
                    bootstrap_method_attr_index: read_u16(&mut reader),
                    name_and_type_index: read_u16(&mut reader),
                },
                18 => ConstantPoolType::ConstantInvokeDynamic {
                    bootstrap_method_attr_index: read_u16(&mut reader),
                    name_and_type_index: read_u16(&mut reader),
                },
                19 => ConstantPoolType::ConstantModule {
                    name_idx: read_u16(&mut reader),
                },
                20 => ConstantPoolType::ConstantPackage {
                    name_idx: read_u16(&mut reader),
                },
                _ => panic!("invalid cp_info tag: {cp_info_tag}"), // todo: or return error?
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

        class_file.access_flags = read_u16(&mut reader);

        class_file.this_class = read_u16(&mut reader);

        class_file.super_class = read_u16(&mut reader);

        class_file.interfaces_count = read_u16(&mut reader);

        for _ in 0..class_file.interfaces_count {
            class_file.interfaces.push(read_u8(&mut reader));
        }

        class_file.fields_count = read_u16(&mut reader);

        for _ in 0..class_file.fields_count {
            class_file.fields.push(read_field_info(&mut reader));
        }

        class_file.methods_count = read_u16(&mut reader);

        for _ in 0..class_file.methods_count {
            class_file.methods.push(read_method_info(&mut reader));
        }

        trace!("class file: {:?}", class_file);

        // TODO: validate class file e.g. indexes into constant pool are valid

        // TODO: print disassembly

        Ok(())
    }
}

fn utf8(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    let len = read_u16(reader);
    let bytes = read_variable(reader, len as usize);
    let value = std::str::from_utf8(&bytes).unwrap().to_string();

    Ok(ConstantPoolType::ConstantUtf8 { len, value })
}

fn long(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    // long
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let value = i64::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantLong { value })
}

fn double(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    // double
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let value = f64::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantDouble { value })
}

fn float(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    // float
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let value = f32::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantFloat { value })
}

fn integer(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    let mut buf = [0u8; 4];
    let _ = reader.read_exact(&mut buf).map_err(|_| {
        Err::<ConstantPoolType, DecompileError>(DecompileError::InvalidInputFile(todo!("get path")))
    });

    let value = i32::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantInteger { value })
}

fn read_u8(reader: &mut BufReader<File>) -> u8 {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    u8::from_be_bytes(buf)
}

fn read_u32(reader: &mut BufReader<File>) -> u32 {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    u32::from_be_bytes(buf)
}

fn read_u16(reader: &mut BufReader<File>) -> u16 {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    u16::from_be_bytes(buf)
}

fn read_variable(reader: &mut BufReader<File>, len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    buf
}

fn read_attribute_info(reader: &mut BufReader<File>) -> AttributeInfo {
    let index = read_u16(reader);
    let length = read_u32(reader);

    AttributeInfo {
        attribute_name_index: index,
        attribute_length: length,
        info: read_variable(reader, length as usize),
    }
}
fn read_field_info(reader: &mut BufReader<File>) -> FieldInfo {
    let access_flags = read_u16(reader);
    let name_index = read_u16(reader);
    let descriptor_index = read_u16(reader);
    let attributes_count = read_u16(reader);

    let mut field_info = FieldInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes_count,
        attributes: vec![],
    };

    for _ in 0..attributes_count {
        field_info.attributes.push(read_attribute_info(reader));
    }

    field_info
}

fn read_method_info(reader: &mut BufReader<File>) -> MethodInfo {
    let access_flags = read_u16(reader);
    let name_index = read_u16(reader);
    let descriptor_index = read_u16(reader);
    let attributes_count = read_u16(reader);

    let mut method_info = MethodInfo {
        access_flags,
        name_index,
        descriptor_index,
        attributes_count,
        attributes: vec![],
    };

    for _ in 0..attributes_count {
        method_info.attributes.push(read_attribute_info(reader));
    }

    method_info
}
