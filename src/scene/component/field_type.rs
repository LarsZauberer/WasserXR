#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub enum FieldType {
    Long,
    Float,
    Char,
    String,

    #[default]
    Blob,
}
