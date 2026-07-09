#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
/// Runtime type hint for a schema field.
pub enum FieldType {
    /// `i8`
    I8,
    /// `i16`
    I16,
    /// `i32`
    I32,
    /// `i64`
    I64,
    /// `i128`
    I128,
    /// `isize`
    Isize,
    /// `u8`
    U8,
    /// `u16`
    U16,
    /// `u32`
    U32,
    /// `u64`
    U64,
    /// `u128`
    U128,
    /// `usize`
    Usize,
    /// `f32`
    F32,
    /// `f64`
    F64,
    /// `[f32; 2]`
    F32Vec2,
    /// `[f32; 3]`
    F32Vec3,
    /// `[f64; 2]`
    F64Vec2,
    /// `[f64; 3]`
    F64Vec3,
    /// `char`
    Char,
    /// `String`
    String,

    #[default]
    /// Opaque data that can be queried by pointer but cannot be text-parsed.
    Blob,
    /// `boolean`
    Boolean,
}
