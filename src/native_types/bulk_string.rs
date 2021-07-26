use std::io::{BufRead, Lines};

use super::{
    error::ErrorStruct,
    redis_type::{verify_parsable_bulk_size, RedisType},
};

/// Redis native type: BulkString
pub struct RBulkString;

impl RedisType<String> for RBulkString {
    fn encode(text: String) -> String {
        if text == "(nil)" {
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

    fn decode<G>(first_lecture: String, buffer: &mut Lines<G>) -> Result<String, ErrorStruct>
    where
        G: BufRead,
    {
        verify_parsable_bulk_size(first_lecture, buffer)
    }
}

#[cfg(test)]
mod test_bulk_string {

    use super::*;
    use std::io::BufReader;
    #[test]
    fn test_04_encoding_of_a_bulk_string() {
        let bulk = String::from("Hello world");
        let encoded = RBulkString::encode(bulk);
        assert_eq!(encoded, "$11\r\nHello world\r\n".to_string());
    }

    #[test]
    fn test_02_bulk_string_decoding() {
        let encoded = RBulkString::encode(String::from("Hello world"));
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let decoded = RBulkString::decode(first_lecture, &mut bufreader.lines());
        assert_eq!(decoded.unwrap(), "Hello world".to_string());
    }
    /*
    #[test]
    fn test_03_bulk_string_decoding_empties_original_string() {
        let mut encoded = RBulkString::encode(String::from("Hello world"));
        encoded.remove(0);
        let _decoded = RBulkString::decode(&mut encoded.as_bytes());
        assert_eq!(encoded, "".to_string());
    }
    */
    #[test]
    fn test_08_bad_decoding_of_bulk_string_throws_a_parsing_error() {
        let encoded = "$Good Morning".to_string();
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let should_be_error = RBulkString::decode(first_lecture, &mut bufreader.lines());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis bulk string".to_string()
                );
            }
        }
        let encoded = "$Good Morning\r\n".to_string();
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let should_be_error = RBulkString::decode(first_lecture, &mut bufreader.lines());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis bulk string".to_string()
                );
            }
        }
        let encoded = "$5\r\nGood Morning\r\n".to_string();
        let mut bufreader = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let should_be_error = RBulkString::decode(first_lecture, &mut bufreader.lines());
        match should_be_error {
            Ok(_string) => {}
            Err(error) => {
                assert_eq!(
                    error.print_it(),
                    "ERR_PARSE Failed to parse redis bulk string".to_string()
                );
            }
        }
    }

    /*
    #[test]
    fn test_10_set_key_value_simulation() {
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

        assert_eq!(
            encoded,
            "$3\r\nSET\r\n$4\r\nping\r\n$4\r\npong\r\n".to_string()
        );

        let mut bulks: Vec<String> = Vec::new();
        for _i in 0..3 {
            match encoded.remove(0) {
                '$' => {
                    let bulk = RBulkString::decode(&mut encoded.as_bytes());
                    bulks.push(bulk.unwrap());
                }
                _ => {}
            }
        }

        assert_eq!(bulks.pop().unwrap(), "pong".to_string());
        assert_eq!(bulks.pop().unwrap(), "ping".to_string());
        assert_eq!(bulks.pop().unwrap(), "SET".to_string());
    }
    */
}
