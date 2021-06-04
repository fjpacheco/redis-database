use std::io::BufRead;

use super::{error::ErrorStruct, redis_type::RedisType};

pub struct RInteger;

impl RedisType<isize> for RInteger {
    fn encode(num: isize) -> String {
        let mut encoded = String::from(":");
        encoded.push_str(&num.to_string());
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(value: &mut dyn BufRead) -> Result<isize, ErrorStruct> {
        let mut sliced_value = String::new();
        match value.read_line(&mut sliced_value) {
            Ok(_) => {
                sliced_value.pop();
                sliced_value.pop();
                if let Ok(integer) = sliced_value.parse::<isize>() {
                    Ok(integer)
                } else {
                    Err(ErrorStruct::new(
                        "ERR_PARSE".to_string(),
                        "Failed to parse redis simple string".to_string(),
                    ))
                }
            }
            Err(_) => Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse redis simple string".to_string(),
            )),
        }
    }
}

#[cfg(test)]
pub mod test_integer {

    use super::*;
    #[test]
    fn test01_encoding_and_decoding_of_an_integer() {
        let integer: isize = 1234;
        let mut encoded = RInteger::encode(integer);
        assert_eq!(encoded, ":1234\r\n".to_string());
        encoded.remove(0);
        let integer_decoded = RInteger::decode(&mut encoded.as_bytes());
        assert_eq!(integer_decoded.unwrap(), 1234);
    }

    #[test]
    fn test02_bad_decoding_of_integer_throws_a_parsing_error() {
        let encoded = "123a\r\n".to_string();
        let should_be_error = RInteger::decode(&mut encoded.as_bytes());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis simple string".to_string()
                );
            }
        }

        let encoded = "123".to_string();
        let should_be_error = RInteger::decode(&mut encoded.as_bytes());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis simple string".to_string()
                );
            }
        }
    }
}
