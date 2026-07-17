use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMap {
    pub name: String,
    pub offset: u32,
    pub data_type: FieldType,
    pub scale: f64,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    U16,
    I16,
    U32,
    I32,
    F32,
    Bitfield(u16),
}

impl FieldMap {
    pub fn new(name: &str, offset: u32, data_type: FieldType) -> Self {
        Self {
            name: name.into(),
            offset,
            data_type,
            scale: 1.0,
            unit: None,
        }
    }
}
