use super::{
    error::ErrorStruct,
    redis_type::{remove_first_cr_lf, RedisType},
};

pub struct RSimpleString;

impl RedisType<String> for RSimpleString {
    fn encode(text: String) -> String {
        let mut encoded = String::from("+");
        encoded.push_str(&text);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(string: &mut String) -> Result<String, ErrorStruct> {
        if let Some(sliced_s_string) = remove_first_cr_lf(string) {
            Ok(sliced_s_string)
        } else {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse redis simple string".to_string(),
            ))
        }
    }
}

#[cfg(test)]
pub mod test_simple_string {

    use super::*;
    #[test]
    fn test01_simple_string_encoding() {
        let simple_string = String::from("word");
        let encoded = RSimpleString::encode(simple_string);
        assert_eq!(encoded, "+word\r\n".to_string());
    }

    #[test]
    fn test02_simple_string_decoding() {
        let mut encoded = "+word\r\n".to_string();
        encoded.remove(0);
        let simple_string = RSimpleString::decode(&mut encoded);
        assert_eq!(simple_string.unwrap(), "word".to_string());
        assert_eq!(encoded, "".to_string());
    }

    #[test]
    fn test03_decoding_of_bad_simple_string_throws_parsing_error() {
        let mut encoded = "Good Morning\r\r".to_string();
        let should_be_error = RSimpleString::decode(&mut encoded);
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
