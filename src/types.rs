/// see https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.1
#[derive(Debug, Default)]
pub struct ClassFile {
    pub magic: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub constant_pool_count: u16,
    pub constant_pool: Vec<CpInfo>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Vec<u8>,
    pub fields_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub enum ConstantPoolType {
    ConstantClass {
        name_idx: u16,
    },
    ConstantFieldref {
        class_index: u16,
        name_and_type_idx: u16,
    },
    ConstantMethodref {
        class_index: u16,
        name_and_type_idx: u16,
    },
    ConstantInterfaceMethodref {
        class_index: u16,
        name_and_type_idx: u16,
    },
    ConstantString {
        string_idx: u16,
    },
    ConstantInteger {
        value: i32,
    },
    ConstantFloat {
        value: f32,
    },
    ConstantLong {
        value: i64,
    },
    ConstantDouble {
        value: f64,
    },
    ConstantNameAndType {
        name_idx: u16,
        desc_idx: u16,
    },
    ConstantUtf8 {
        // TODO: do we need to keep len?
        len: u16, // the number of bytes to read in the class file  (not the length of the resulting string).
        value: String,
    },
    ConstantMethodHandle {
        ref_kind: u8,
        ref_idx: u16,
    },
    ConstantMethodType {
        desc_idx: u16,
    },
    ConstantDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    ConstantInvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    ConstantModule {
        name_idx: u16,
    },
    ConstantPackage {
        name_idx: u16,
    },
}

#[derive(Debug, Default)]
pub struct CpInfo {
    pub tag: u8,
    pub info: Option<ConstantPoolType>,
}

#[derive(Debug, Default)]
pub struct FieldAccessFlags {}

#[derive(Debug, Default)]
pub struct FieldInfo {
    // pub access_flags: FieldAccessFlags,
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Default)]
pub struct MethodAccessFlags {}

#[derive(Debug, Default)]
pub struct MethodInfo {
    // pub access_flags: MethodAccessFlags,
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Default)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub info: Vec<u8>,
}
