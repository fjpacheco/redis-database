use super::{error::ErrorStruct, redis_type::{RedisType, remove_first_cr_lf, verify_parsable_bulk_size}};

pub struct RBulkString;

impl RedisType<String> for RBulkString {

    fn encode(text: String) -> String {
        if text == "(nil)".to_string() {
            "$-1\r\n".to_string()
        } else {
            let mut encoded = String::from("$");
            encoded.push_str(&(text.len()).to_string());
            encoded.push('\r');
            encoded.push('\n');
            encoded.push_str(&text);
            encoded.push('\r');
            encoded.push('\n');
            encoded
        }
    }

    fn decode(bulk: &mut String) -> Result<String,ErrorStruct> {
        if let Some(sliced_size) = remove_first_cr_lf(bulk) {
            verify_parsable_bulk_size(sliced_size, bulk)
        } else {
            Err(ErrorStruct::new("ERR_PARSE".to_string(), "Failed to parse redis bulk string".to_string()))
        }
    }

}


#[cfg(test)]
mod test_bulk_string {

    use super::*;
    #[test]
    fn test01_bulk_string_enconding() {
        let bulk = String::from("Hello world");
        let encoded = RBulkString::encode(bulk);
        assert_eq!(encoded, "$11\r\nHello world\r\n".to_string());
    }
    
    #[test]
    fn test02_bulk_string_decoding() {
        let mut encoded = RBulkString::encode(String::from("Hello world"));
        encoded.remove(0);
        let decoded = RBulkString::decode(&mut encoded);
        assert_eq!(decoded.unwrap(), "Hello world".to_string());
        assert_eq!(encoded, "".to_string());
    }
    #[test]
    fn test03_bulk_string_decoding_empties_original_string() {
        let mut encoded = RBulkString::encode(String::from("Hello world"));
        encoded.remove(0);
        let _decoded = RBulkString::decode(&mut encoded);
        assert_eq!(encoded, "".to_string());
    }

    #[test]
    fn test04_bulk_string_empty_string_encoding() {
        let encoded = RBulkString::encode(String::from(""));
        // Averiguar si es correcto considerar este caso
        assert_eq!(encoded, "$0\r\n\r\n".to_string());
    }

    #[test]
    fn test05_bulk_string_empty_string_decoding() {
        let mut encoded = RBulkString::encode(String::from(""));
        encoded.remove(0);
        let decoded = RBulkString::decode(&mut encoded);
        assert_eq!(decoded.unwrap(), "".to_string());
    }

    #[test]
    fn test06_bulk_string_nil_encoding() {
        let encoded = RBulkString::encode(String::from("(nil)"));
        assert_eq!(encoded, "$-1\r\n".to_string());
    }

    #[test]
    fn test07_bulk_string_nil_decoding() {
        let mut encoded = RBulkString::encode(String::from("(nil)"));
        encoded.remove(0);
        let decoded = RBulkString::decode(&mut encoded);
        assert_eq!(decoded.unwrap(), "(nil)".to_string());
    }

    #[test]
    fn test08_wrong_bulk_string_decoding_throws_parsing_error() {
        let mut encoded = "$Good Morning".to_string();
        let should_be_error = RBulkString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis bulk string".to_string());
            }
        }
        let mut encoded = "$Good Morning\r\n".to_string();
        let should_be_error = RBulkString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis bulk string".to_string());
            }
        }
        let mut encoded = "$5\r\nGood Morning\r\n".to_string();
        let should_be_error = RBulkString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis bulk string".to_string());
            }
        }
    }

    #[test]
    fn test09_set_key_value_simulation() {
        let input = "SET ping pong";
        let mut v: Vec<&str> = input.rsplit(' ').collect();
        let command = v.pop().unwrap().to_string();
        let key = v.pop().unwrap().to_string();
        let value = v.pop().unwrap().to_string();
        assert_eq!(command, "SET".to_string());
        assert_eq!(key, "ping".to_string());
        assert_eq!(value, "pong".to_string());

        let mut encoded = RBulkString::encode(command);
        encoded.push_str(&RBulkString::encode(key));
        encoded.push_str(&RBulkString::encode(value));

        assert_eq!(encoded, "$3\r\nSET\r\n$4\r\nping\r\n$4\r\npong\r\n".to_string());

        let mut bulks: Vec<String> = Vec::new();
        for _i in 0..3{
            match encoded.remove(0) {
                '$' => {
                    let bulk = RBulkString::decode(&mut encoded);
                    bulks.push(bulk.unwrap());
                },
                _ => {},
            }
        }

        assert_eq!(bulks.pop().unwrap(), "pong".to_string());
        assert_eq!(bulks.pop().unwrap(), "ping".to_string());
        assert_eq!(bulks.pop().unwrap(), "SET".to_string());
        assert!(encoded.is_empty());
    }

}