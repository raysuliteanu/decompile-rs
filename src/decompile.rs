use crate::error::DecompileError;
use crate::error::DecompileError::{InvalidMagicNumber, NoSuchFile};
use crate::types::{
    Attribute, ClassFile, ConstantPoolType, CpInfo, ExceptionTable, FieldInfo, InnerClassInfo,
    LineNumberTableEntry, MethodInfo, MethodParameter,
};
use log::{debug, trace};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
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

        let constant_pool_count = read_u16(&mut reader);

        debug!("constant pool count: {}", constant_pool_count);

        for _ in 0..constant_pool_count - 2 {
            let pos = reader.stream_position()?;
            let cp_info_tag = read_u8(&mut reader);
            let cp_info_type = match cp_info_tag {
                1 => cp_utf8(&mut reader)?,
                3 => cp_integer(&mut reader)?,
                4 => cp_float(&mut reader)?,
                5 => cp_long(&mut reader)?,
                6 => cp_double(&mut reader)?,
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
                _ => {
                    debug!("class_file:\n{class_file}");
                    return Err(DecompileError::InvalidConstantPoolTag(cp_info_tag, pos));
                }
            };

            let info = CpInfo {
                tag: cp_info_tag,
                info: Some(cp_info_type),
            };

            class_file.add_constant_pool_entry(info);
        }

        debug!(
            "read {} constant pool items",
            class_file.get_constant_pool_size()
        );

        class_file.access_flags = read_u16(&mut reader);
        debug!("access_flags: {:#x}", class_file.access_flags);

        class_file.this_class = read_u16(&mut reader);
        debug!("this_class idx: {}", class_file.this_class);

        class_file.super_class = read_u16(&mut reader);
        debug!("super_class idx: {}", class_file.super_class);

        class_file.interfaces_count = read_u16(&mut reader);
        debug!("interfaces_count: {}", class_file.interfaces_count);

        for _ in 0..class_file.interfaces_count {
            let value = read_u8(&mut reader);
            debug!("interface idx: {value}");
            class_file.interfaces.push(value);
        }

        class_file.fields_count = read_u16(&mut reader);
        debug!("fields_count: {}", class_file.fields_count);

        for _ in 0..class_file.fields_count {
            let field_info = read_field_info(&mut reader, &class_file)?;
            debug!("adding {:?}", field_info);
            class_file.fields.push(field_info);
        }

        class_file.methods_count = read_u16(&mut reader);
        debug!("methods_count: {}", class_file.methods_count);

        for _ in 0..class_file.methods_count {
            let method_info = read_method_info(&mut reader, &class_file)?;
            debug!("adding {:?}", method_info);
            class_file.methods.push(method_info);
        }

        trace!("class file: {:?}", class_file);

        // TODO: validate class file e.g. indexes into constant pool are valid
        // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.8

        // TODO: print disassembly

        Ok(())
    }
}

fn cp_utf8(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("cp_utf8()");

    let len = read_u16(reader);
    let bytes = read_variable(reader, len as usize);
    debug!("utf8: len({len}) bytes: {:x?}", bytes);
    let value = std::str::from_utf8(&bytes).unwrap().to_string();

    Ok(ConstantPoolType::ConstantUtf8 { len, value })
}

fn cp_integer(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("cp_integer()");
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let value = i32::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantInteger { value })
}

fn cp_long(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("cp_long()");
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let value = i64::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantLong { value })
}

fn cp_float(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("cp_float()");
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let value = f32::from_be_bytes(buf);

    Ok(ConstantPoolType::ConstantFloat { value })
}

fn cp_double(reader: &mut BufReader<File>) -> DecompileResult<ConstantPoolType> {
    trace!("cp_double()");
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

fn read_field_info(
    reader: &mut BufReader<File>,
    class_file: &ClassFile,
) -> DecompileResult<FieldInfo> {
    trace!("read_field_info()");
    let access_flags = read_u16(reader);
    let name_index = read_u16(reader);
    let descriptor_index = read_u16(reader);
    let attributes_count = read_u16(reader);
    debug!("field_info: name_index {name_index} descriptor_index {descriptor_index} attributes_count {attributes_count}");

    let field_name = resolve_utf8_cp_entry(reader, class_file, name_index)?;
    debug!("resolved field name: {field_name}");
    let field_descriptor = resolve_utf8_cp_entry(reader, class_file, descriptor_index)?;
    debug!("resolved descriptor: {field_descriptor}");

    let mut field_info = FieldInfo {
        access_flags,
        name: field_name,
        descriptor: field_descriptor,
        value: None,
        attributes: Vec::new(),
    };

    for _ in 0..attributes_count {
        let attr = read_attribute_info(reader, class_file)?;
        match attr {
            Attribute::ConstantValue {
                constant_value_index,
                ..
            } => {
                // constantvalue_index
                //     The value of the constantvalue_index item must be a valid index into the constant_pool
                //     table. The constant_pool entry at that index gives the value represented by this attribute.
                //     The constant_pool entry must be of a type appropriate to the field,

                // TODO: check that the field is a static field
                if field_info.value.is_some() {
                    todo!("only one value allowed, so this is an error");
                }

                field_info.value = if let Some(cp_info) =
                    &class_file.get_constant_pool_entry(constant_value_index as usize)
                {
                    match &cp_info.info {
                        Some(ConstantPoolType::ConstantDouble { value }) => {
                            Some(format!("{value}"))
                        }
                        Some(ConstantPoolType::ConstantFloat { value }) => Some(format!("{value}")),
                        Some(ConstantPoolType::ConstantLong { value }) => Some(format!("{value}")),
                        Some(ConstantPoolType::ConstantInteger { value }) => {
                            Some(format!("{value}"))
                        }
                        Some(ConstantPoolType::ConstantString { string_idx }) => {
                            if let Some(info) =
                                class_file.get_constant_pool_entry(*string_idx as usize)
                            {
                                if let Some(ConstantPoolType::ConstantUtf8 { value, len: _ }) =
                                    &info.info
                                {
                                    Some(value.clone())
                                } else {
                                    return Err(DecompileError::InvalidUtf8ConstantPoolEntry(
                                        *string_idx,
                                    ));
                                }
                            } else {
                                return Err(DecompileError::NoSuchConstantPoolEntry(
                                    name_index,
                                    reader.stream_position()?,
                                ));
                            }
                        }
                        _ => todo!("invalid field value type"),
                    }
                } else {
                    return Err(DecompileError::NoSuchConstantPoolEntry(
                        name_index,
                        reader.stream_position()?,
                    ));
                };
            }
            // TODO: handle other attributes
            _ => debug!("ignoring attribute {:?}", attr),
        }

        debug!("adding attribute {:?}", attr);
        field_info.attributes.push(attr);
    }

    Ok(field_info)
}

fn read_method_info(
    reader: &mut BufReader<File>,
    class_file: &ClassFile,
) -> DecompileResult<MethodInfo> {
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
            .push(read_attribute_info(reader, class_file)?);
    }

    Ok(method_info)
}

fn read_attribute_info(
    reader: &mut BufReader<File>,
    class_file: &ClassFile,
) -> DecompileResult<Attribute> {
    trace!("read_attribute_info()");

    let index = read_u16(reader);
    let length = read_u32(reader);

    debug!("attr_info: index {index} len {length}");

    let attr_name = resolve_utf8_cp_entry(reader, class_file, index)?;
    debug!("resolved attr name: {attr_name}");

    let attr = match attr_name.as_str() {
        "ConstantValue" => {
            // attribute_length
            //     The value of the attribute_length item must be two.
            assert_eq!(length, 2);
            let constant_value_index = read_u16(reader);
            Attribute::ConstantValue {
                attribute_name_index: index,
                attribute_length: length,
                constant_value_index,
            }
        }
        "Code" => {
            let max_stack = read_u16(reader);
            let max_locals = read_u16(reader);
            let code_length = read_u32(reader);
            let code = read_variable(reader, code_length as usize);
            let exception_table_length = read_u16(reader);
            let mut exception_table = Vec::with_capacity(exception_table_length as usize);
            for _ in 0..exception_table_length {
                exception_table.push(ExceptionTable {
                    start_pc: read_u16(reader),
                    end_pc: read_u16(reader),
                    handler_pc: read_u16(reader),
                    catch_type: read_u16(reader),
                })
            }
            let attributes_count = read_u16(reader);
            let mut attributes = Vec::with_capacity(attributes_count as usize);
            for _ in 0..attributes_count {
                attributes.push(read_attribute_info(reader, class_file)?);
            }
            Attribute::Code {
                attribute_name_index: index,
                attribute_length: length,
                max_stack,
                max_locals,
                code_length,
                code,
                exception_table_length,
                exception_table,
                attributes_count,
                attributes,
            }
        }
        "LineNumberTable" => {
            let line_number_table_length = read_u16(reader);
            let mut line_number_table = Vec::with_capacity(line_number_table_length as usize);
            for _ in 0..line_number_table_length {
                line_number_table.push(LineNumberTableEntry {
                    start_pc: read_u16(reader),
                    line_number: read_u16(reader),
                });
            }

            Attribute::LineNumberTable {
                attribute_name_index: index,
                attribute_length: length,
                line_number_table_length,
                line_number_table,
            }
        }
        "SourceFile" => Attribute::SourceFile {
            attribute_name_index: index,
            attribute_length: length,
            sourcefile_index: read_u16(reader),
        },
        "MethodParameters" => {
            let parameters_count = read_u8(reader);
            let mut parameters = Vec::with_capacity(parameters_count as usize);
            for _ in 0..parameters_count {
                parameters.push(MethodParameter {
                    name_index: read_u16(reader),
                    access_flags: read_u16(reader),
                });
            }
            Attribute::MethodParameters {
                attribute_name_index: index,
                attribute_length: length,
                parameters_count,
                parameters,
            }
        }
        "InnerClasses" => {
            let number_of_classes = read_u16(reader);
            let mut classes = Vec::with_capacity(number_of_classes as usize);
            for _ in 0..number_of_classes {
                classes.push(InnerClassInfo {
                    inner_class_info_index: read_u16(reader),
                    outer_class_info_index: read_u16(reader),
                    inner_name_index: read_u16(reader),
                    inner_class_access_flags: read_u16(reader),
                })
            }
            Attribute::InnerClasses {
                attribute_name_index: index,
                attribute_length: length,
                number_of_classes,
                classes,
            }
        }
        // "StackMapTable" => {}
        // "Exceptions" => {}
        // "EnclosingMethod" => {}
        // "Synthetic" => {}
        // "Signature" => {}
        // "SourceDebugExtension" => {}
        // "LocalVariableTable" => {}
        // "LocalVariableTypeTable" => {}
        // "Deprecated" => {}
        // "Module" => {}
        // "ModulePackages" => {}
        // "ModuleMainClass" => {}
        // "Record" => {}
        // "PermittedSubclasses" => {}
        _ => todo!("ignoring {attr_name} for now"),
    };

    debug!("adding attr: {:?}", attr);

    Ok(attr)
}

fn resolve_utf8_cp_entry(
    reader: &mut BufReader<File>,
    class_file: &ClassFile,
    index: u16,
) -> DecompileResult<String> {
    let value = if let Some(cp_info) = class_file.get_constant_pool_entry(index as usize) {
        if let Some(ConstantPoolType::ConstantUtf8 { value, len: _ }) = &cp_info.info {
            value.clone()
        } else {
            return Err(DecompileError::InvalidUtf8ConstantPoolEntry(index));
        }
    } else {
        return Err(DecompileError::NoSuchConstantPoolEntry(
            index,
            reader.stream_position()?,
        ));
    };

    debug!("resolved utf8 entry: {value}");

    Ok(value)
}
