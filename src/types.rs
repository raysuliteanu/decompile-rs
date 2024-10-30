use std::fmt::Display;
use std::fmt::Write;

use log::debug;

/// see https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.1
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct ClassFile {
    pub magic: u32,
    pub major_version: u16,
    pub minor_version: u16,

    constant_pool: ConstantPool,

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
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Default)]
pub struct ConstantPool {
    // The value of the constant_pool_count item is equal to the number of
    // entries in the constant_pool table plus one. A constant_pool index is
    // considered valid if it is greater than zero and less than
    // constant_pool_count, with the exception for constants of type long and
    // double noted in
    // https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.4.5
    cp_info: Vec<CpInfo>,
}

impl ConstantPool {
    fn push(&mut self, cp_info: CpInfo) {
        self.cp_info.push(cp_info)
    }

    fn len(&self) -> usize {
        self.cp_info.len()
    }

    fn get(&self, idx: usize) -> Option<&CpInfo> {
        self.cp_info.get(idx)
    }
}

impl Display for ConstantPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .cp_info
            .iter()
            .enumerate()
            .fold(String::new(), |mut s, (i, v)| {
                let _ = writeln!(s, "idx: {:0>2} entry: {{ {v} }}", i + 1);
                s
            });

        write!(f, "{s}")
    }
}

impl ClassFile {
    pub(crate) fn new(magic: u32) -> Self {
        ClassFile {
            magic,
            ..Self::default()
        }
    }

    pub(crate) fn add_constant_pool_entry(&mut self, cp_info: CpInfo) {
        debug!("adding {:?} at {}", cp_info, self.constant_pool.len() + 1);
        self.constant_pool.push(cp_info);
    }

    pub(crate) fn get_constant_pool_size(&self) -> usize {
        self.constant_pool.len()
    }

    // See https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.1
    // constant_pool[]
    //      "The constant_pool table is indexed from 1 to constant_pool_count - 1."
    pub(crate) fn get_constant_pool_entry(&self, index: usize) -> Option<&CpInfo> {
        self.constant_pool.get(index - 1)
    }
}

impl Display for ClassFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "magic: {:x}\nversion: {}.{}\ncp_count: {}\ncp: [\n{}\n]\naccess_flags: {:x}\nthis_class: {}",
            self.magic, self.major_version, self.minor_version, self.constant_pool.len(), self.constant_pool, self.access_flags, self.this_class
        )?;
        writeln!(f)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConstantPoolType {
    ConstantClass {
        name_idx: u16,
    },
    ConstantFieldRef {
        class_index: u16,
        name_and_type_idx: u16,
    },
    ConstantMethodRef {
        class_index: u16,
        name_and_type_idx: u16,
    },
    ConstantInterfaceMethodRef {
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

impl Display for ConstantPoolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstantPoolType::ConstantClass { name_idx } => write!(f, "Class: ({name_idx})"),
            ConstantPoolType::ConstantFieldRef {
                class_index,
                name_and_type_idx,
            } => write!(
                f,
                "FieldRef: class({class_index}) name_and_type({name_and_type_idx})"
            ),
            ConstantPoolType::ConstantMethodRef {
                class_index,
                name_and_type_idx,
            } => write!(
                f,
                "MethodRef: class({class_index}) name_and_type({name_and_type_idx})"
            ),
            ConstantPoolType::ConstantInterfaceMethodRef {
                class_index,
                name_and_type_idx,
            } => write!(
                f,
                "InterfaceMethodRef: class({class_index}) name_and_type({name_and_type_idx})"
            ),
            ConstantPoolType::ConstantString { string_idx } => write!(f, "String: ({string_idx})"),
            ConstantPoolType::ConstantInteger { value } => write!(f, "Integer: {value}"),
            ConstantPoolType::ConstantFloat { value } => write!(f, "Float: {value}"),
            ConstantPoolType::ConstantLong { value } => write!(f, "Long: {value}"),
            ConstantPoolType::ConstantDouble { value } => write!(f, "Double: {value}"),
            ConstantPoolType::ConstantNameAndType { name_idx, desc_idx } => {
                write!(f, "NameAndType: name({name_idx}) desc({desc_idx})")
            }
            ConstantPoolType::ConstantUtf8 { len, value } => {
                write!(f, "Utf8: len({len}) value(\"{value}\")")
            }
            ConstantPoolType::ConstantMethodHandle { ref_kind, ref_idx } => {
                write!(f, "MethodHandle: ref_kind({ref_kind}) ref_idx({ref_idx}")
            }
            ConstantPoolType::ConstantMethodType { desc_idx } => {
                write!(f, "MethodType: desc({desc_idx})")
            }
            ConstantPoolType::ConstantDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => write!(f, "Dynamic: bootstrap_method_attr({bootstrap_method_attr_index}) name_and_type({name_and_type_index})"),
            ConstantPoolType::ConstantInvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => write!(f, "InvokeDynamic: bootstrap_method_attr({bootstrap_method_attr_index}) name_and_type({name_and_type_index})"),
            ConstantPoolType::ConstantModule { name_idx } => write!(f, "Module: ({name_idx})"),
            ConstantPoolType::ConstantPackage { name_idx } => write!(f, "Package: ({name_idx})"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct CpInfo {
    pub tag: u8,
    pub info: Option<ConstantPoolType>,
}

impl Display for CpInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = if let Some(cp_type) = &self.info {
            format!("{cp_type}")
        } else {
            "None".to_string()
        };

        write!(f, "tag: {:0>2} info: {{ {info} }}", self.tag)
    }
}

#[derive(Debug, Default)]
pub struct FieldAccessFlags {}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct FieldInfo {
    // pub access_flags: FieldAccessFlags,
    pub access_flags: u16,
    /*
        pub name_index: u16,
        pub descriptor_index: u16,
        pub attributes_count: u16,
    */
    pub name: String,
    pub descriptor: String,
    pub value: Option<String>,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Default)]
pub struct MethodAccessFlags {}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct MethodInfo {
    // pub access_flags: MethodAccessFlags,
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<Attribute>,
}

// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7
#[allow(dead_code)]
#[derive(Debug)]
pub enum Attribute {
    //https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.2
    ConstantValue {
        attribute_name_index: u16,
        attribute_length: u32,
        constant_value_index: u16,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.3
    Code {
        attribute_name_index: u16,
        attribute_length: u32,
        max_stack: u16,
        max_locals: u16,
        code_length: u32,
        code: Vec<u8>,
        exception_table_length: u16,
        exception_table: Vec<ExceptionTable>,
        attributes_count: u16,
        attributes: Vec<Attribute>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.4
    StackMapTable {
        attribute_name_index: u16,
        attribute_length: u32,
        number_of_entries: u16,
        entries: Vec<StackMapFrame>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.5
    Exceptions {
        attribute_name_index: u16,
        attribute_length: u32,
        number_of_exceptions: u16,
        exception_index_table: Vec<u16>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.6
    InnerClasses {
        attribute_name_index: u16,
        attribute_length: u32,
        number_of_classes: u16,
        classes: Vec<InnerClassInfo>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.7
    EnclosingMethod {
        attribute_name_index: u16,
        attribute_length: u32,
        class_index: u16,
        method_index: u16,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.8
    Synthetic {
        attribute_name_index: u16,
        attribute_length: u32,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.9
    Signature {
        attribute_name_index: u16,
        attribute_length: u32,
        signature_index: u16,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.10
    SourceFile {
        attribute_name_index: u16,
        attribute_length: u32,
        sourcefile_index: u16,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.11
    SourceDebugExtension {
        attribute_name_index: u16,
        attribute_length: u32,
        debug_extension: Vec<u8>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.12
    LineNumberTable {
        attribute_name_index: u16,
        attribute_length: u32,
        line_number_table_length: u16,
        line_number_table: Vec<LineNumberTableEntry>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.13
    LocalVariableTable {
        attribute_name_index: u16,
        attribute_length: u32,
        local_variable_table_length: u16,
        local_variable_table: Vec<LocalVariableTableEntry>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.14
    LocalVariableTypeTable {
        attribute_name_index: u16,
        attribute_length: u32,
        local_variable_type_table_length: u16,
        local_variable_type_table: Vec<LocalVariableTypeTableEntry>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.15
    Deprecated {
        attribute_name_index: u16,
        attribute_length: u32,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.16
    RuntimeVisibleAnnotations {
        attribute_name_index: u16,
        attribute_length: u32,
        num_annotations: u16,
        annotations: Vec<Annotation>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.17
    RuntimeInvisibleAnnotations {
        attribute_name_index: u16,
        attribute_length: u32,
        num_annotations: u16,
        annotations: Vec<Annotation>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.18
    RuntimeVisibleParameterAnnotations {
        attribute_name_index: u16,
        attribute_length: u32,
        num_parameters: u8,
        parameter_annotations: Vec<Annotation>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.19
    RuntimeInvisibleParameterAnnotations {
        attribute_name_index: u16,
        attribute_length: u32,
        num_parameters: u8,
        parameter_annotations: Vec<Annotation>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.20
    RuntimeVisibleTypeAnnotations {
        attribute_name_index: u16,
        attribute_length: u32,
        num_annotations: u16,
        annotations: Vec<TypeAnnotation>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.21
    RuntimeInvisibleTypeAnnotations {
        attribute_name_index: u16,
        attribute_length: u32,
        num_annotations: u16,
        annotations: Vec<TypeAnnotation>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.22
    AnnotationDefault {
        attribute_name_index: u16,
        attribute_length: u32,
        default_value: ElementValue,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.23
    BootstrapMethods {
        attribute_name_index: u16,
        attribute_length: u32,
        num_bootstrap_methods: u16,
        bootstrap_methods: Vec<BootstrapMethod>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.24
    MethodParameters {
        attribute_name_index: u16,
        attribute_length: u32,
        parameters_count: u8,
        parameters: Vec<MethodParameter>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.25
    Module {
        attribute_name_index: u16,
        attribute_length: u32,
        module_name_index: u16,
        module_flags: u16,
        module_version_index: u16,
        requires_count: u16,
        requires: Vec<ModuleRequirement>,
        exports_count: u16,
        exports: Vec<ModuleExport>,
        opens_count: u16,
        opens: Vec<ModuleOpens>,
        uses_count: u16,
        uses_index: Vec<u16>,
        provides_count: u16,
        provides: Vec<ModuleProvides>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.26
    ModulePackages {
        attribute_name_index: u16,
        attribute_length: u32,
        package_count: u16,
        package_index: Vec<u16>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.27
    ModuleMainClass {
        attribute_name_index: u16,
        attribute_length: u32,
        main_class_index: u16,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.28
    NestHost {
        attribute_name_index: u16,
        attribute_length: u32,
        host_class_index: u16,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.29
    NestMembers {
        attribute_name_index: u16,
        attribute_length: u32,
        number_of_classes: u16,
        classes: Vec<u16>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.30
    Record {
        attribute_name_index: u16,
        attribute_length: u32,
        component_count: u16,
        components: Vec<RecordComponentInfo>,
    },
    // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.31
    PermittedSubclasses {
        attribute_name_index: u16,
        attribute_length: u32,
        number_of_classes: u16,
        classes: Vec<u16>,
    },
}

impl From<String> for Attribute {
    fn from(value: String) -> Self {
        match value {
            _ => panic!("invalid attribute name"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RecordComponentInfo {
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<Attribute>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModuleProvides {
    provides_index: u16,
    provides_with_count: u16,
    provides_with_index: Vec<u16>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModuleOpens {
    opens_index: u16,
    opens_flags: u16,
    opens_to_count: u16,
    opens_to_index: Vec<u16>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModuleRequirement {
    requires_index: u16,
    requires_flags: u16,
    requires_version_index: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ModuleExport {
    exports_index: u16,
    exports_flags: u16,
    exports_to_count: u16,
    exports_to_index: Vec<u16>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BootstrapMethod {
    bootstrap_method_ref: u16,
    num_bootstrap_arguments: u16,
    bootstrap_arguments: Vec<u16>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TypeAnnotation {
    target_type: u8,
    target_info: TargetInfo,
    target_path: TypePath,
    type_index: u16,
    num_element_value_pairs: u16,
    element_value_pairs: Vec<AnnotationElementPair>,
}

// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.20.1
#[allow(dead_code)]
#[derive(Debug)]
pub enum TargetInfo {
    TypeParameter(u8),
    SuperType(u16),
    TypeParameterBound {
        type_parameter_index: u8,
        bound_index: u8,
    },
    Empty,
    FormalParameter(u8),
    Throws(u16),
    LocalVar {
        table_length: u16,
        table: Vec<LocalVarTable>,
    },
    Catch(u16),
    Offset(u16),
    TypeArgument {
        offset: u16,
        type_argument_index: u8,
    },
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LocalVarTable {
    start_pc: u16,
    length: u16,
    index: u16,
}

// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.20.2
#[allow(dead_code)]
#[derive(Debug)]
pub struct TypePath {
    path_length: u8,
    path: Vec<TypePathElement>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TypePathElement {
    type_path_kind: u8,
    type_argument_index: u8,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Annotation {
    type_index: u16,
    num_element_value_pairs: u16,
    element_value_pairs: Vec<AnnotationElementPair>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AnnotationElementPair {
    element_name_index: u16,
    value: ElementValue,
}

// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.16.1
#[allow(dead_code)]
#[derive(Debug)]
pub enum ElementValue {
    ConstValueIndex(u16),
    EnumConstantValue {
        type_name_index: u16,
        const_name_index: u16,
    },
    ClassInfoIndex(u16),
    AnnotationValue(Annotation),
    ArrayValue {
        num_values: u16,
        values: Vec<ElementValue>,
    },
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LocalVariableTypeTableEntry {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LocalVariableTableEntry {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LineNumberTableEntry {
    pub(crate) start_pc: u16,
    pub(crate) line_number: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InnerClassInfo {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StackMapFrame {
    SameFrame,
    SameLocals1StackItemFrame,
    SameLocals1StackItemFrameExtended,
    ChopFrame,
    SameFrameExtended,
    AppendFrame,
    FullFrame,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExceptionTable {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}
