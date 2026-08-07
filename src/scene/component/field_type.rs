#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
/// Runtime type hint for a schema field.
pub enum FieldType {
    /// `i8`
    I8 = 0,
    /// `i16`
    I16 = 1,
    /// `i32`
    I32 = 2,
    /// `i64`
    I64 = 3,
    /// `i128`
    I128 = 4,
    /// `isize`
    Isize = 5,
    /// `u8`
    U8 = 6,
    /// `u16`
    U16 = 7,
    /// `u32`
    U32 = 8,
    /// `u64`
    U64 = 9,
    /// `u128`
    U128 = 10,
    /// `usize`
    Usize = 11,
    /// `f32`
    F32 = 12,
    /// `f64`
    F64 = 13,
    /// `[f32; 2]`
    F32Vec2 = 14,
    /// `[f32; 3]`
    F32Vec3 = 15,
    /// `[f64; 2]`
    F64Vec2 = 16,
    /// `[f64; 3]`
    F64Vec3 = 17,
    /// `char`
    Char = 18,
    /// `String`
    String = 19,

    #[default]
    /// Opaque data that can be queried by pointer but cannot be text-parsed.
    Blob = 20,
    /// `boolean`
    Boolean = 21,
}

impl TryFrom<u32> for FieldType {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::I8),
            1 => Ok(Self::I16),
            2 => Ok(Self::I32),
            3 => Ok(Self::I64),
            4 => Ok(Self::I128),
            5 => Ok(Self::Isize),
            6 => Ok(Self::U8),
            7 => Ok(Self::U16),
            8 => Ok(Self::U32),
            9 => Ok(Self::U64),
            10 => Ok(Self::U128),
            11 => Ok(Self::Usize),
            12 => Ok(Self::F32),
            13 => Ok(Self::F64),
            14 => Ok(Self::F32Vec2),
            15 => Ok(Self::F32Vec3),
            16 => Ok(Self::F64Vec2),
            17 => Ok(Self::F64Vec3),
            18 => Ok(Self::Char),
            19 => Ok(Self::String),
            20 => Ok(Self::Blob),
            21 => Ok(Self::Boolean),
            value => Err(value),
        }
    }
}
