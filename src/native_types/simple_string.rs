use std::io::{BufRead, Lines};

use super::{error::ErrorStruct, redis_type::RedisType};

/// Redis native type: Simple String
pub struct RSimpleString;

impl RedisType<String> for RSimpleString {
    fn encode(text: String) -> String {
        let mut encoded = String::from("+");
        encoded.push_str(&text);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode<G>(
        first_lecture: String,
        _redis_encoded_line: &mut Lines<G>,
    ) -> Result<String, ErrorStruct>
    where
        G: BufRead,
    {
        Ok(first_lecture)
    }
}

#[cfg(test)]
pub mod test_simple_string {

    use super::*;
    use std::io::BufReader;
    #[test]
    fn test01_simple_string_encoding() {
        let simple_string = String::from("word");
        let encoded = RSimpleString::encode(simple_string);
        assert_eq!(encoded, "+word\r\n".to_string());
    }

    #[test]
    fn test02_simple_string_decoding() {
        let encoded = "+word\r\n".to_string();
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let simple_string = RSimpleString::decode(first_lecture, &mut bufreader.lines());
        assert_eq!(simple_string.unwrap(), "word".to_string());
    }

    /*#[test]
    fn test03_bad_decoding_of_simple_string_throws_a_parsing_error() {
        let encoded = "Good Morning".to_string();
        let should_be_error = RSimpleString::decode(&mut encoded.as_bytes());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis simple buffer".to_string()
                );
            }
        }
    }*/
}
