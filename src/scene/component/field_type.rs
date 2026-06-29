#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub enum FieldType {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    F32Vec2,
    F32Vec3,
    F64Vec2,
    F64Vec3,
    Char,
    String,

    #[default]
    Blob,
}
