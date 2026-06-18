#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum FieldType {
    Long,
    Float,
    Char,
    String,
    Blob,
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::Blob
    }
}
