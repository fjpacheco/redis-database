use std::io::{BufRead, Lines};

use super::{error::ErrorStruct, redis_type::RedisType};

/// Redis native type: Integer
pub struct RInteger;

impl RedisType<isize> for RInteger {
    fn encode(num: isize) -> String {
        let mut encoded = String::from(":");
        encoded.push_str(&num.to_string());
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode<G>(
        first_lecture: String,
        _redis_encoded_line: &mut Lines<G>,
    ) -> Result<isize, ErrorStruct>
    where
        G: BufRead,
    {
        if let Ok(integer) = first_lecture.parse::<isize>() {
            Ok(integer)
        } else {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse redis integer".to_string(),
            ))
        }
    }
}

#[cfg(test)]
pub mod test_integer {

    use super::*;
    use std::io::BufReader;
    #[test]
    fn test_01_encoding_and_decoding_of_an_integer() {
        let integer: isize = 1234;
        let encoded = RInteger::encode(integer);
        assert_eq!(encoded, ":1234\r\n".to_string());
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let integer_decoded = RInteger::decode(first_lecture, &mut bufreader.lines());
        assert_eq!(integer_decoded.unwrap(), 1234);
    }

    #[test]
    fn test_02_bad_decoding_of_integer_throws_a_parsing_error() {
        let encoded = "+123a\r\n".to_string();
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let should_be_error = RInteger::decode(first_lecture, &mut bufreader.lines());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis integer".to_string()
                );
            }
        }
    }
}
