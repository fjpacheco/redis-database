#[derive(Debug, Clone)]
pub enum HttpUrl {
    Path(String),
}

/*impl HttpUrl {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub fn get(&self) -> String {
        String::from(&self.url)
    }
}*/
