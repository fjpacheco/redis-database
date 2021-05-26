use std::collections::{HashMap, HashSet};

// Mock DatabaseMock
pub struct DatabaseMock {
    strings: HashMap<String, String>,
    lists: HashMap<String, Vec<String>>,
    sets: HashMap<String, HashSet<String>>,
}

impl DatabaseMock {
    pub fn new() -> Self {
        DatabaseMock {
            strings: HashMap::new(),
            lists: HashMap::new(),
            sets: HashMap::new(),
        }
    }

    // No funcionó ... :C
    pub fn get_mut_strings(&mut self) -> &mut HashMap<String, String> {
        &mut self.strings
    }

    // No funcionó ... :C
    pub fn get_mut_lists(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.lists
    }

    // No funcionó ... :C
    pub fn get_mut_sets(&mut self) -> &mut HashMap<String, HashSet<String>> {
        &mut self.sets
    }

    #[allow(clippy::all)] // Y me quejaba clippy :C
    pub fn get_mut_fields(
        &mut self,
    ) -> (
        &mut HashMap<String, String>,
        &mut HashMap<String, Vec<String>>,
        &mut HashMap<String, HashSet<String>>,
    ) {
        (&mut self.strings, &mut self.lists, &mut self.sets)
    }
}

impl Default for DatabaseMock {
    fn default() -> Self {
        Self::new()
    }
}
