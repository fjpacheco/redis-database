fn remove_first_cr_lf(slice: &mut String) -> Option<String> {

    if let Some(first_cr) = slice.find('\r') {
        if slice.remove(first_cr + 1) == '\n' {

            slice.remove(first_cr);
            let rest = slice.split_off(first_cr);
            let swap = slice.to_string();
            *slice = rest; 
            
            Some(swap)

        } else {
            None
        }
    } else {
        None
    }

}

#[derive(Debug)]
pub struct Error {

    prefix: String,
    message: String

}

impl Error {

    pub fn new(prefix: String, message: String) -> Self {
        Error{prefix, message}
    }

    #[allow(dead_code)]
    pub fn print_it(&self) -> String{
        let mut printed = self.prefix.to_string();
        printed.push(' ');
        printed.push_str(&self.message.to_string());
        printed
    }

}

pub trait RedisType<T>{
    fn encode(t: T) -> String;
    fn decode(redis_encoded_line: &mut String) -> Result<T, Error>;
}

pub struct RSimpleString;
pub struct RError;
pub struct RInteger;
pub struct RBulkString;

impl RedisType<String> for RSimpleString {

    fn encode(text: String) -> String {
        let mut encoded = String::from("+");
        encoded.push_str(&text);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(string: &mut String) -> Result<String, Error> {
        if let Some(sliced_s_string) = remove_first_cr_lf(string) {
            Ok(sliced_s_string)
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }
    }

}

impl RedisType<Error> for RError {

    fn encode(err: Error) -> String {
        let mut encoded = String::from("-");
        encoded.push_str(&err.prefix);
        encoded.push(' ');
        encoded.push_str(&err.message);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(error: &mut String) -> Result<Error, Error> {
        if let Some(mut sliced_error) = remove_first_cr_lf(error) {
            if let Some(middle_space) = sliced_error.find(' '){

                let err_message = sliced_error.split_off(middle_space + 1);
                sliced_error.pop();
                Ok(Error{
                    prefix: sliced_error,
                    message: err_message,
                })

            } else {
                Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
            }
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }
    }

}

impl RedisType<isize> for RInteger {

    fn encode(num: isize) -> String {
        let mut encoded = String::from(":");
        encoded.push_str(&num.to_string());
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(value: &mut String) -> Result<isize, Error> {
        if let Some(sliced_value) = remove_first_cr_lf(value) {
            if let Ok(integer) = sliced_value.parse::<isize>() {
                Ok(integer)
            } else {
                Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
            }
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }

    }

}

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

    fn decode(bulk: &mut String) -> Result<String, Error>{
        if let Some(sliced_size) = remove_first_cr_lf(bulk) {
            verify_that_size_is_parsable(sliced_size, bulk)
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }
    }

}

#[allow(dead_code)]
    fn verify_that_size_is_parsable(sliced_size: String, rest_of: &mut String) -> Result<String, Error>{
        if let Ok(size) = sliced_size.parse::<isize>() {

            if size == -1 {
                Ok("(nil)".to_string())
            } else {
                split_b_string(size, rest_of)
            }
            
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }

    }

    #[allow(dead_code)]
    fn split_b_string(size: isize, rest_of: &mut String) -> Result<String, Error> {
        if let Some(sliced_b_string) = remove_first_cr_lf(rest_of){
            verify_size_of_b_string(size, sliced_b_string)
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }

    }

    #[allow(dead_code)]
    fn verify_size_of_b_string(size: isize, sliced_b_string: String) -> Result<String, Error>{
        if sliced_b_string.len() == size as usize{
            Ok(sliced_b_string)
        } else {
            Err(Error::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }

    }

    
#[cfg(test)]
mod test_decode {

    use super::*;
    #[test]
    fn test01_encoding_of_a_simple_string() {
        let simple_string = String::from("word");
        let encoded = RSimpleString::encode(simple_string);
        assert_eq!(encoded, "+word\r\n".to_string());
    }

    #[test]
    fn test02_decoding_of_a_simple_string() {
        let mut encoded = "word\r\n".to_string();
        let simple_string = RSimpleString::decode(&mut encoded);
        assert_eq!(simple_string.unwrap(), "word".to_string());
        assert_eq!(encoded, "".to_string());
    }

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
    fn test04_encoding_and_decoding_of_a_bulk_string() {

        let bulk = String::from("Hello world");
        let mut encoded = RBulkString::encode(bulk);
        assert_eq!(encoded, "$11\r\nHello world\r\n".to_string());
        encoded.remove(0);
        let integer_decoded = RBulkString::decode(&mut encoded);
        assert_eq!(integer_decoded.unwrap(), "Hello world".to_string());
        assert_eq!(encoded, "".to_string());

    }

    #[test]
    fn test05_encoding_and_decoding_of_an_error() {

        let error = Error::new("ERR".to_string(), "esto es un error generico".to_string());
        let mut encoded = RError::encode(error);
        assert_eq!(encoded, "-ERR esto es un error generico\r\n".to_string());
        encoded.remove(0);
        let error_decoded = RError::decode(&mut encoded);
        assert_eq!(error_decoded.unwrap().print_it(), "ERR esto es un error generico".to_string());
        assert_eq!(encoded, "".to_string());

    }

    #[test]
    fn test06_bad_decoding_of_simple_string_throws_a_parsing_error(){

        let mut encoded = "Good Morning".to_string();
        let should_be_error = RSimpleString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis simple string".to_string());
            }
        }

    }

    #[test]
    fn test07_bad_decoding_of_integer_throws_a_parsing_error(){

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

    #[test]
    fn test08_bad_decoding_of_bulk_string_throws_a_parsing_error(){

        let mut encoded = "$Good Morning".to_string();
        let should_be_error = RBulkString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis simple string".to_string());
            }
        }

        let mut encoded = "$Good Morning\r\n".to_string();
        let should_be_error = RBulkString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis simple string".to_string());
            }
        }

        let mut encoded = "$5\r\nGood Morning\r\n".to_string();
        let should_be_error = RBulkString::decode(&mut encoded);
        match should_be_error {
            Ok(_string) => {},
            Err(error) => {
                assert_eq!(error.print_it(), "ERR_PARSE Failed to parse redis simple string".to_string());
            }
        }

    }

    /*#[test]
    fn test09_unknown_redis_char_type_throws_a_unknown_type_error(){
        let encoded = "%Good Morning".to_string();
        let (should_be_error, _) = NativeTypes::new(encoded);
        assert_eq!(should_be_error.get().unwrap(), "ERR_UNKNOWN_TYPE Failed to match the first byte. Unknown Redis type".to_string());
    }*/

    #[test]
    fn test10_set_key_value_simulation() {

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

    /*#[test]
    fn test11_nil(){

        let should_be_nil = NativeTypes::new_nil();
        assert_eq!(should_be_nil.get().unwrap(), "(nil)".to_string());

        let encoded_nil = should_be_nil.encode().unwrap();
        assert_eq!(encoded_nil, "$-1\r\n".to_string());

        let (decoded_nil, encoded_nil) = NativeTypes::new(encoded_nil);
        assert_eq!(decoded_nil.get().unwrap(), "(nil)".to_string());
        assert_eq!(encoded_nil, "".to_string());
    
    }*/

    
}