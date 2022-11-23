use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rectangle {
    pub(crate) x1: u32,
    pub(crate) y1: u32,

    pub(crate) x2: u32,
    pub(crate) y2: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AlfalfaData {
    pub version: u8,
    pub data: HashMap<String, Vec<u8>>,
}
