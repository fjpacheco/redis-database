use std::collections::{HashMap, HashSet};

pub struct DatabaseMock2 {
    elements: HashMap<String, TypeSaved>,
}

#[derive(Debug, PartialEq)]
pub enum TypeSaved {
    String(String),
    Lists(Vec<String>),
    Sets(HashSet<String>), // old versión
}

impl DatabaseMock2 {
    pub fn new() -> Self {
        DatabaseMock2 {
            elements: HashMap::new(),
        }
    }

    pub fn get_mut_elements(&mut self) -> &mut HashMap<String, TypeSaved> {
        &mut self.elements
    }
}

impl Default for DatabaseMock2 {
    fn default() -> Self {
        Self::new()
    }
}
