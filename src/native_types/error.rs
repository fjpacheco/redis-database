use std::io::{BufRead, Lines};

use super::redis_type::RedisType;

#[derive(Debug, Clone)]
pub struct ErrorStruct {
    prefix: String,
    message: String,
}

impl ErrorStruct {
    pub fn new(prefix: String, message: String) -> Self {
        ErrorStruct { prefix, message }
    }
    #[allow(dead_code)]
    pub fn print_it(&self) -> String {
        let mut printed = self.prefix.to_string();
        printed.push(' ');
        printed.push_str(&self.message.to_string());
        printed
    }

    // Para tests... investigar si existe una macro así: #[metodo_para_test]
    pub fn get_encoded_message_complete(&self) -> String {
        RError::encode(self.clone())
    }
}
pub struct RError;

impl RedisType<ErrorStruct> for RError {
    fn encode(err: ErrorStruct) -> String {
        let mut encoded = String::from("-");
        encoded.push_str(&err.prefix);
        encoded.push(' ');
        encoded.push_str(&err.message);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode<G>(
        mut first_lecture: String,
        _redis_encoded_line: &mut Lines<G>,
    ) -> Result<ErrorStruct, ErrorStruct>
    where
        G: BufRead,
    {
        if let Some(middle_space) = first_lecture.find(' ') {
            let err_message = first_lecture.split_off(middle_space + 1);
            first_lecture.pop();
            Ok(ErrorStruct {
                prefix: first_lecture,
                message: err_message,
            })
        } else {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse redis error".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod test_error {

    use super::*;
    use std::io::BufReader;
    #[test]
    fn test05_encoding_and_decoding_of_an_error() {
        let error = ErrorStruct::new("ERR".to_string(), "esto es un error generico".to_string());
        let encoded = RError::encode(error);
        assert_eq!(encoded, "-ERR esto es un error generico\r\n".to_string());
        let mut buffer = BufReader::new(encoded.as_bytes());
        let mut first_lecture = String::new();
        buffer.read_line(&mut first_lecture).unwrap();
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        let error_decoded = RError::decode(first_lecture, &mut buffer.lines());
        assert_eq!(
            error_decoded.unwrap().print_it(),
            "ERR esto es un error generico".to_string()
        );
    }
}
