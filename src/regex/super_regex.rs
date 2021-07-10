use regex::Error;
use regex::Regex;

pub struct SuperRegex {
    matcher: Regex,
}

impl SuperRegex {
    pub fn from(input: &str) -> Result<SuperRegex, Error> {
        let mut regex = String::from("^");
        let replaced = input.replace("?", ".").replace("*", ".*");
        regex.push_str(&replaced);
        regex.push('$');
        let matcher = Regex::new(&regex)?;
        Ok(SuperRegex { matcher })
    }

    pub fn is_match(&self, word: &str) -> bool {
        self.matcher.is_match(word)
    }
}

#[cfg(test)]
pub mod test_super_regex {

    use super::*;

    #[test]
    pub fn test01_one_character() {
        let regex = SuperRegex::from("h?llo").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(regex.is_match("hillo"));
        assert!(!regex.is_match("heello"));
        assert!(!regex.is_match("helloaa"));
        assert!(!regex.is_match("aahello"));
        assert!(!regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test02_many_character() {
        let regex = SuperRegex::from("h*llo").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(regex.is_match("hillo"));
        assert!(regex.is_match("heello"));
        assert!(!regex.is_match("helloaa"));
        assert!(!regex.is_match("aahello"));
        assert!(!regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test03_left_side_characters() {
        let regex = SuperRegex::from("*hello").unwrap();
        assert!(regex.is_match("hello"));
        assert!(!regex.is_match("hallo"));
        assert!(!regex.is_match("hillo"));
        assert!(!regex.is_match("heello"));
        assert!(!regex.is_match("helloaa"));
        assert!(regex.is_match("aahello"));
        assert!(!regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test04_right_side_characters() {
        let regex = SuperRegex::from("hello*").unwrap();
        assert!(regex.is_match("hello"));
        assert!(!regex.is_match("hallo"));
        assert!(!regex.is_match("hillo"));
        assert!(!regex.is_match("heello"));
        assert!(regex.is_match("helloaa"));
        assert!(!regex.is_match("aahello"));
        assert!(!regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test05_all() {
        let regex = SuperRegex::from("*").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(regex.is_match("buenassss"));
        assert!(regex.is_match("heello"));
        assert!(regex.is_match("papita"));
        assert!(regex.is_match("martina?"));
        assert!(regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test06_matches_with_ae() {
        let regex = SuperRegex::from("h[ae]llo").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(!regex.is_match("hillo"));
        assert!(!regex.is_match("heello"));
        assert!(!regex.is_match("helloaa"));
        assert!(!regex.is_match("aahello"));
        assert!(!regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test07_doesnt_matches_with_i() {
        let regex = SuperRegex::from("h[^i]llo").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(!regex.is_match("hillo"));
        assert!(!regex.is_match("heello"));
        assert!(!regex.is_match("helloaa"));
        assert!(regex.is_match("hpllo"));
        assert!(!regex.is_match("aahelloaa"));
    }

    #[test]
    pub fn test08_matches_in_a_range() {
        let regex = SuperRegex::from("h[a-f]llo").unwrap();
        assert!(regex.is_match("hallo"));
        assert!(regex.is_match("hbllo"));
        assert!(regex.is_match("hcllo"));
        assert!(regex.is_match("hdllo"));
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hfllo"));
        assert!(!regex.is_match("hgllo"));
    }
}
