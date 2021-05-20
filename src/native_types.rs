#[derive(Debug)]
pub enum NativeTypes {
    SimpleString(String),
    Integer(isize),
    BulkString(usize, String),
    Error(String, String),
}

impl NativeTypes {
    #[allow(dead_code)]
    pub fn new(mut redis_string: String) -> (Self, String) {
        match redis_string.remove(0) {
            '+' => Self::new_simple_string_from(redis_string),
            ':' => Self::new_integer_from(redis_string),
            '$' => Self::new_bulk_string_from(redis_string),
            '-' => Self::new_error_from(redis_string),
            _ => Self::new_error_from(
                "ERR_UNKNOWN_TYPE Failed to match the first byte. Unknown Redis type\r\n"
                    .to_string(),
            ),
        }
    }

    #[allow(dead_code)]
    fn new_simple_string_from(string: String) -> (Self, String) {
        if let Some((sliced_s_string, rest_of)) = NativeTypes::remove_first_cr_lf(string) {
            (Self::SimpleString(sliced_s_string), rest_of)
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis simple string\r\n".to_string())
        }
    }

    #[allow(dead_code)]
    fn new_integer_from(value: String) -> (Self, String) {
        if let Some((sliced_value, rest_of)) = NativeTypes::remove_first_cr_lf(value) {
            if let Ok(integer) = sliced_value.parse::<isize>() {
                (Self::Integer(integer), rest_of)
            } else {
                Self::new_error_from("ERR_PARSE Failed to parse redis integer\r\n".to_string())
            }
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis integer\r\n".to_string())
        }
    }

    #[allow(dead_code)]
    fn new_bulk_string_from(bulk: String) -> (Self, String) {
        let sliced_size: String;
        let rest_of: String;
        if let Some((a, b)) = NativeTypes::remove_first_cr_lf(bulk) {
            sliced_size = a;
            rest_of = b;
        } else {
            return Self::new_error_from(
                "ERR_PARSE Failed to parse redis bulk string\r\n".to_string(),
            );
        };

        let size: usize;
        if let Ok(a) = sliced_size.parse::<usize>() {
            size = a;
        } else {
            return Self::new_error_from(
                "ERR_PARSE Failed to parse redis bulk string\r\n".to_string(),
            );
        };

        let sliced_b_string: String;
        let rest_of2: String;
        if let Some((a, b)) = NativeTypes::remove_first_cr_lf(rest_of) {
            sliced_b_string = a;
            rest_of2 = b;
        } else {
            return Self::new_error_from(
                "ERR_PARSE Failed to parse redis bulk string\r\n".to_string(),
            );
        };

        if sliced_b_string.len() == size {
            println!("HOla");
            (Self::BulkString(size, sliced_b_string), rest_of2)
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis bulk string\r\n".to_string())
        }
    }

    /*fn verify_that_size_is_parsable(sliced_size: String, rest_of: String) -> (Self, String){
        if let Ok(size) = sliced_size.parse::<usize>() {
            println!("HOla");
            NativeTypes::split_b_string(size, rest_of)
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis simple string\r\n".to_string())
        }
    }

    fn split_b_string(size: usize, rest_of: String) -> (Self, String) {
        if let Some((sliced_b_string, rest_of)) = NativeTypes::remove_first_cr_lf(rest_of){
            println!("HOla");
            NativeTypes::verify_size_of_b_string(size, sliced_b_string, rest_of)
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis simple string\r\n".to_string())
        }

    }

    fn verify_size_of_b_string(size: usize, sliced_b_string: String, rest_of: String) -> (Self, String) {
        if sliced_b_string.len() == size {
            println!("HOla");
            (Self::BulkString(size, sliced_b_string), rest_of)
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis simple string\r\n".to_string())
        }
    }*/

    #[allow(dead_code)]
    fn remove_first_cr_lf(mut slice: String) -> Option<(String, String)> {
        if let Some(first_cr) = slice.find('\r') {
            if slice.remove(first_cr + 1) == '\n' {
                slice.remove(first_cr);
                let rest = slice.split_off(first_cr);
                Some((slice, rest))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn new_error_from(error: String) -> (Self, String) {
        if let Some((mut sliced_error, rest_of)) = NativeTypes::remove_first_cr_lf(error) {
            if let Some(middle_space) = sliced_error.find(' ') {
                let err_message = sliced_error.split_off(middle_space + 1);
                sliced_error.pop();
                (Self::Error(sliced_error, err_message), rest_of)
            } else {
                Self::new_error_from("ERR_PARSE Failed to parse redis error\r\n".to_string())
            }
        } else {
            Self::new_error_from("ERR_PARSE Failed to parse redis error\r\n".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn new_simple_string(str: &str) -> Self {
        Self::SimpleString(str.to_string())
    }

    #[allow(dead_code)]
    pub fn new_integer(str: &str) -> Self {
        Self::Integer(str.parse::<isize>().unwrap())
    }

    #[allow(dead_code)]
    pub fn new_bulk_string(str: &str) -> Self {
        let string = str.to_string();
        Self::BulkString(string.len(), string)
    }

    #[allow(dead_code)]
    pub fn new_error(str: &str) -> Self {
        let mut error = str.to_string();
        let middle_point = error.find(' ').unwrap();
        let err_message = error.split_off(middle_point + 1);
        error.pop();
        Self::Error(error, err_message)
    }

    #[allow(dead_code)]
    pub fn encode(&self) -> Option<String> {
        match &self {
            Self::SimpleString(text) => Some(NativeTypes::encode_simple_string(text.to_string())),
            Self::Integer(num) => Some(NativeTypes::encode_integer(*num)),
            Self::BulkString(size, bulk) => {
                Some(NativeTypes::encode_bulk_string(*size, bulk.to_string()))
            }
            Self::Error(prefix, message) => Some(NativeTypes::encode_error(
                prefix.to_string(),
                message.to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    fn encode_simple_string(text: String) -> String {
        let mut encoded = String::from("+");
        encoded.push_str(&text);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    #[allow(dead_code)]
    fn encode_integer(num: isize) -> String {
        let mut encoded = String::from(":");
        encoded.push_str(&num.to_string());
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    #[allow(dead_code)]
    fn encode_bulk_string(size: usize, bulk: String) -> String {
        let mut encoded = String::from("$");
        encoded.push_str(&size.to_string());
        encoded.push('\r');
        encoded.push('\n');
        encoded.push_str(&bulk);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    #[allow(dead_code)]
    fn encode_error(err_prefix: String, err_message: String) -> String {
        let mut encoded = String::from("-");
        encoded.push_str(&err_prefix);
        encoded.push(' ');
        encoded.push_str(&err_message);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    #[allow(dead_code)]
    pub fn get(&self) -> Option<String> {
        match &self {
            Self::SimpleString(text) => Some(text.to_string()),
            Self::Integer(num) => Some(num.to_string()),
            Self::BulkString(_, bulk) => Some(bulk.to_string()),
            Self::Error(prefix, message) => Some(NativeTypes::error_to_string(
                prefix.to_string(),
                message.to_string(),
            )),
        }
    }

    #[allow(dead_code)]
    fn error_to_string(mut prefix: String, message: String) -> String {
        prefix.push(' ');
        prefix.push_str(&message);
        prefix
    }
}

#[cfg(test)]
mod test_decode {

    use super::*;
    #[test]
    fn test01_encoding_of_a_simple_string() {
        let simple_string = NativeTypes::new_simple_string("word");
        assert_eq!(simple_string.get().unwrap(), "word".to_string());
        let encoded = simple_string.encode().unwrap();
        assert_eq!(encoded, "+word\r\n".to_string());
    }

    #[test]
    fn test02_decoding_of_a_simple_string() {
        let encoded = "+word\r\n".to_string();
        let (simple_string, rest_of) = NativeTypes::new(encoded);
        assert_eq!(simple_string.get().unwrap(), "word".to_string());
        assert_eq!(rest_of, "".to_string());
    }

    #[test]
    fn test03_encoding_and_decoding_of_an_integer() {
        let integer = NativeTypes::new_integer("1234");
        assert_eq!(integer.get().unwrap(), "1234".to_string());
        let encoded = integer.encode().unwrap();
        assert_eq!(encoded, ":1234\r\n".to_string());
        let (integer_decoded, rest_of) = NativeTypes::new(encoded);
        assert_eq!(integer_decoded.get().unwrap(), "1234".to_string());
        assert_eq!(rest_of, "".to_string());
    }

    #[test]
    fn test04_encoding_and_decoding_of_a_bulk_string() {
        let bulk_string = NativeTypes::new_bulk_string("Hello world");
        assert_eq!(bulk_string.get().unwrap(), "Hello world".to_string());
        let encoded = bulk_string.encode().unwrap();
        assert_eq!(encoded, "$11\r\nHello world\r\n".to_string());
        let (bulk_string_decoded, rest_of) = NativeTypes::new(encoded);
        assert_eq!(
            bulk_string_decoded.get().unwrap(),
            "Hello world".to_string()
        );
        assert_eq!(rest_of, "".to_string());
    }

    #[test]
    fn test05_encoding_and_decoding_of_an_error() {
        let error = NativeTypes::new_error("ERR esto es un error generico");
        assert_eq!(
            error.get().unwrap(),
            "ERR esto es un error generico".to_string()
        );
        let encoded = error.encode().unwrap();
        assert_eq!(encoded, "-ERR esto es un error generico\r\n".to_string());
        let (error_decoded, rest_of) = NativeTypes::new(encoded);
        assert_eq!(
            error_decoded.get().unwrap(),
            "ERR esto es un error generico".to_string()
        );
        assert_eq!(rest_of, "".to_string());
    }

    #[test]
    fn test06_bad_decoding_of_simple_string_throws_a_parsing_error() {
        let encoded = "+Good Morning".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_PARSE Failed to parse redis simple string".to_string()
        );
    }

    #[test]
    fn test07_bad_decoding_of_integer_throws_a_parsing_error() {
        let encoded = ":123a\r\n".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_PARSE Failed to parse redis integer".to_string()
        );

        let encoded = ":123".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_PARSE Failed to parse redis integer".to_string()
        );
    }

    #[test]
    fn test08_bad_decoding_of_bulk_string_throws_a_parsing_error() {
        let encoded = "$Good Morning".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_PARSE Failed to parse redis bulk string".to_string()
        );

        let encoded = "$Good Morning\r\n".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_PARSE Failed to parse redis bulk string".to_string()
        );

        let encoded = "$5\r\nGood Morning\r\n".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_PARSE Failed to parse redis bulk string".to_string()
        );
    }

    #[test]
    fn test09_unknown_redis_char_type_throws_a_unknown_type_error() {
        let encoded = "%Good Morning".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(
            should_be_error.get().unwrap(),
            "ERR_UNKNOWN_TYPE Failed to match the first byte. Unknown Redis type".to_string()
        );
    }

    #[test]
    fn test10_set_key_value_simulation() {
        let input = "SET ping pong";
        let mut v: Vec<&str> = input.rsplit(' ').collect();
        let command = v.pop().unwrap();
        let key = v.pop().unwrap();
        let value = v.pop().unwrap();
        assert_eq!(command, "SET");
        assert_eq!(key, "ping");
        assert_eq!(value, "pong");

        let bulk_command = NativeTypes::new_bulk_string(command);
        let bulk_key = NativeTypes::new_bulk_string(key);
        let bulk_value = NativeTypes::new_bulk_string(value);

        let mut encoded = bulk_command.encode().unwrap();
        encoded.push_str(&bulk_key.encode().unwrap());
        encoded.push_str(&bulk_value.encode().unwrap());

        assert_eq!(
            encoded,
            "$3\r\nSET\r\n$4\r\nping\r\n$4\r\npong\r\n".to_string()
        );

        let (decoded_command, encoded) = NativeTypes::new(encoded);
        let (decoded_key, encoded) = NativeTypes::new(encoded);
        let (decoded_value, encoded) = NativeTypes::new(encoded);

        assert_eq!(decoded_command.get().unwrap(), "SET".to_string());
        assert_eq!(decoded_key.get().unwrap(), "ping".to_string());
        assert_eq!(decoded_value.get().unwrap(), "pong".to_string());
        assert!(encoded.is_empty());
    }
}
