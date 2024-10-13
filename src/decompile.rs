use crate::error::DecompileError;
use crate::error::DecompileError::{InvalidMagicNumber, NoSuchFile};
use crate::types::{
    Attribute, ClassFile, ConstantPoolType, CpInfo, FieldAccessFlags, FieldInfo, MethodAccessFlags,
    MethodInfo,
};
use log::{debug, trace};
use std::arch::x86_64::__cpuid_count;
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
                _ => return Err(DecompileError::InvalidClassPoolTag(cp_info_tag)),
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
            class_file
                .fields
                .push(read_field_info(&mut reader, &class_file));
        }

        class_file.methods_count = read_u16(&mut reader);

        for _ in 0..class_file.methods_count {
            class_file
                .methods
                .push(read_method_info(&mut reader, &class_file));
        }

        trace!("class file: {:?}", class_file);

        // TODO: validate class file e.g. indexes into constant pool are valid
        // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.8

        // TODO: print disassembly

        Ok(())
    }
}

fn utf8(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("utf8()");

    let len = read_u16(reader);
    let bytes = read_variable(reader, len as usize);
    let value = std::str::from_utf8(&bytes).unwrap().to_string();

    Ok(ConstantPoolType::ConstantUtf8 { len, value })
}

fn integer(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("integer()");
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let value = i32::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantInteger { value })
}

fn long(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("long()");
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let value = i64::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantLong { value })
}

fn float(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("float()");
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let value = f32::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantFloat { value })
}

fn double(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("double()");
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let value = f64::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantDouble { value })
}

fn read_u8(reader: &mut BufReader<File>) -> u8 {
    trace!("read_utf8()");
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    u8::from_be_bytes(buf)
}

fn read_u16(reader: &mut BufReader<File>) -> u16 {
    trace!("read_u16()");
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    u16::from_be_bytes(buf)
}

fn read_u32(reader: &mut BufReader<File>) -> u32 {
    trace!("read_u32()");
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    u32::from_be_bytes(buf)
}

fn read_variable(reader: &mut BufReader<File>, len: usize) -> Vec<u8> {
    trace!("read_variable({len})");
    let mut buf = vec![0; len];
    reader.read_exact(&mut buf).expect("invalid class file"); // todo: better error
    buf
}

fn read_field_info(reader: &mut BufReader<File>, class_file: &ClassFile) -> FieldInfo {
    trace!("read_field_info()");
    let access_flags = read_u16(reader);
    let name_index = read_u16(reader);
    let descriptor_index = read_u16(reader);
    let attributes_count = read_u16(reader);

    let field_name =
        if let Some(name_info) = class_file.get_constant_pool_entry(name_index as usize) {
            if let Some(ConstantPoolType::ConstantUtf8 { value, len: _ }) = &name_info.info {
                value.clone()
            } else {
                todo!("invalid class file error");
            }
        } else {
            todo!("invalid class file error");
        };

    let field_descriptor =
        if let Some(desc_info) = class_file.get_constant_pool_entry(descriptor_index as usize) {
            if let Some(ConstantPoolType::ConstantUtf8 { value, len: _ }) = &desc_info.info {
                value.clone()
            } else {
                todo!("invalid class file error");
            }
        } else {
            todo!("invalid class file error");
        };

    let (attribute_name_index, attribute_length, constant_value_index) =
        (read_u16(reader), read_u32(reader), read_u16(reader));

    // attribute_length
    //     The value of the attribute_length item must be two.
    assert_eq!(attribute_length, 2);

    // attribute_name_index
    //     The value of the attribute_name_index item must be a valid index into the constant_pool table.
    //     The constant_pool entry at that index must be a CONSTANT_Utf8_info structure (§4.4.7)
    //     representing the string "ConstantValue".
    let constant_value =
        if let Some(cp_info) = class_file.constant_pool.get(attribute_name_index as usize) {
            if let Some(ConstantPoolType::ConstantUtf8 { value, len: _ }) = &cp_info.info {
                value.clone()
            } else {
                todo!("invalid class file error");
            }
        } else {
            todo!("invalid class file error");
        };

    assert_eq!(constant_value, "ConstantValue".to_string());

    // constantvalue_index
    //     The value of the constantvalue_index item must be a valid index into the constant_pool
    //     table. The constant_pool entry at that index gives the value represented by this attribute.
    //     The constant_pool entry must be of a type appropriate to the field,
    let value =
        if let Some(cp_info) = &class_file.get_constant_pool_entry(constant_value_index as usize) {
            // TODO: validate type against descriptor
            match &cp_info.info {
                Some(ConstantPoolType::ConstantDouble { value }) => format!("{value}"),
                Some(ConstantPoolType::ConstantFloat { value }) => format!("{value}"),
                Some(ConstantPoolType::ConstantLong { value }) => format!("{value}"),
                Some(ConstantPoolType::ConstantInteger { value }) => format!("{value}"),
                Some(ConstantPoolType::ConstantString { string_idx }) => {
                    if let Some(info) = class_file.get_constant_pool_entry(*string_idx as usize) {
                        if let Some(ConstantPoolType::ConstantUtf8 { value, len: _ }) = &info.info {
                            value.clone()
                        } else {
                            todo!("")
                        }
                    } else {
                        todo!("")
                    }
                }
                _ => todo!("invalid"),
            }
        } else {
            todo!("")
        };

    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.2
    // -->  There may be at most one ConstantValue attribute in the attributes table of a field_info structure.
    FieldInfo {
        access_flags,
        name: field_name,
        descriptor: field_descriptor,
        value,
        attributes: vec![Attribute::ConstantValue {
            attribute_name_index,
            attribute_length,
            constant_value_index,
        }],
    }
}

fn read_method_info(reader: &mut BufReader<File>, class_file: &ClassFile) -> MethodInfo {
    trace!("read_method_info()");
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
        method_info
            .attributes
            .push(read_attribute_info(reader, class_file));
    }

    method_info
}

fn read_attribute_info(reader: &mut BufReader<File>, class_file: &ClassFile) -> Attribute {
    let index = read_u16(reader);
    let length = read_u32(reader);

    Attribute::ConstantValue {
        attribute_name_index: index,
        attribute_length: length,
        constant_value_index: 0,
    }
}
