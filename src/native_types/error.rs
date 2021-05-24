use super::redis_type::{RedisType, remove_first_cr_lf};

#[derive(Debug)]
pub struct ErrorStruct {
    prefix: String,
    message: String
}

impl ErrorStruct {
    pub fn new(prefix: String, message: String) -> Self {
       ErrorStruct{prefix, message}
    }
    #[allow(dead_code)]
    pub fn print_it(&self) -> String {
        let mut printed = self.prefix.to_string();
        printed.push(' ');
        printed.push_str(&self.message.to_string());
        printed
    }
}
pub struct RError;

impl RedisType<ErrorStruct> for RError {

    fn encode(err:ErrorStruct) -> String {
        let mut encoded = String::from("-");
        encoded.push_str(&err.prefix);
        encoded.push(' ');
        encoded.push_str(&err.message);
        encoded.push('\r');
        encoded.push('\n');
        encoded
    }

    fn decode(error: &mut String) -> Result<ErrorStruct,ErrorStruct> {
        if let Some(mut sliced_error) = remove_first_cr_lf(error) {
            if let Some(middle_space) = sliced_error.find(' '){

                let err_message = sliced_error.split_off(middle_space + 1);
                sliced_error.pop();
                Ok(ErrorStruct {
                    prefix: sliced_error,
                    message: err_message,
                })

            } else {
                Err(ErrorStruct::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
            }
        } else {
            Err(ErrorStruct::new("ERR_PARSE".to_string(), "Failed to parse redis simple string".to_string()))
        }
    }

}

#[cfg(test)]
mod test_error {

    use super::*;
    #[test]
    fn test01_encoding_and_decoding_of_an_error() {
        let error = ErrorStruct::new("ERR".to_string(), "esto es un error generico".to_string());
        let mut encoded = RError::encode(error);
        assert_eq!(encoded, "-ERR esto es un error generico\r\n".to_string());
        encoded.remove(0);
        let error_decoded = RError::decode(&mut encoded);
        assert_eq!(error_decoded.unwrap().print_it(), "ERR esto es un error generico".to_string());
        assert_eq!(encoded, "".to_string());

    }

}

