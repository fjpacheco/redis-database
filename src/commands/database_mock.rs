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

    pub fn get_mut_strings(&mut self) -> &mut HashMap<String, String> {
        &mut self.strings
    }

    pub fn get_mut_lists(&mut self) -> &mut HashMap<String, Vec<String>> {
        &mut self.lists
    }

    pub fn get_mut_sets(&mut self) -> &mut HashMap<String, HashSet<String>> {
        &mut self.sets
    }

    pub fn remove(&mut self, key: &str) {
        self.strings.remove(key);
        self.lists.remove(key);
        self.sets.remove(key);
    }
}

impl Default for DatabaseMock {
    fn default() -> Self {
        Self::new()
    }
}
