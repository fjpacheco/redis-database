use super::{error::ErrorStruct, redis_type::{RedisType, remove_first_cr_lf}};

pub struct RInteger;

impl RedisType<isize> for RInteger {

    fn encode(num: isize) -> String {
        let mut encoded = String::from(":");
        encoded.push_str(&num.to_string());
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(value: &mut String) -> Result<isize,ErrorStruct> {
        if let Some(sliced_value) = remove_first_cr_lf(value) {
            if let Ok(integer) = sliced_value.parse::<isize>() {
                Ok(integer)
            } else {
                Err(ErrorStruct::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
            }
        } else {
            Err(ErrorStruct::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }

    }

}

#[cfg(test)]
pub mod test_integer {

    use super::*;
    #[test]
    fn test03_encoding_and_decoding_of_an_integer() {
        let integer: isize = 1234;
        let mut encoded = RInteger::encode(integer);
        assert_eq!(encoded, ":1234\r\n".to_string());
        encoded.remove(0);
        let integer_decoded = RInteger::decode(&mut encoded);
        assert_eq!(integer_decoded.unwrap(), 1234);
        assert_eq!(encoded, "".to_string());

    }

    #[test]
    fn test07_bad_decoding_of_integer_throws_a_parsing_error() {

        let mut encoded = "123a\r\n".to_string();
        let should_be_error = RInteger::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis simple string".to_string());
            }
        }

        let mut encoded = "123".to_string();
        let should_be_error = RInteger::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis simple string".to_string());
            }
        }

    }

}