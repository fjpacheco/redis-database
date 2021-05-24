use super::{
    bulk_string::RBulkString,
    error::ErrorStruct,
    redis_type::{remove_first_cr_lf, verify_parsable_array_size, RedisType},
};

pub struct RArray;

impl RedisType<Vec<String>> for RArray {
    fn encode(array: Vec<String>) -> String {
        if array.is_empty() {
            "*0\r\n".to_string() // Revisar si podria recibirse empty list
        } else {
            let mut encoded = String::from("*");
            encoded.push_str(&(array.len()).to_string());
            encoded.push('\r');
            encoded.push('\n');
            // Se guardan los elementos como bulks
            for elem in array {
                let encoded_elem = RBulkString::encode(elem);
                encoded.push_str(&encoded_elem);
            }
            encoded
        }
    }

    fn decode(bulk_array: &mut String) -> Result<Vec<String>, ErrorStruct> {
        // 2 \r\n$ 3\r\nfoo\r\n $ 3\r\nbar\r\n
        if let Some(sliced_size) = remove_first_cr_lf(bulk_array) {
            verify_parsable_array_size(&sliced_size, bulk_array)
        } else {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse redis simple string".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod test_array {

    use super::*;
    #[test]
    fn test01_array_encoding() {
        let vec: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        let encoded = RArray::encode(vec);
        assert_eq!(encoded, "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_string());
    }

    #[test]
    fn test02_array_decoding() {
        let vec1: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        let vec2 = vec1.clone();
        let mut encoded: String = RArray::encode(vec1);
        encoded.remove(0); // Saco el '*'
        let decoded = RArray::decode(&mut encoded).unwrap();
        for i in 0..(vec2.len()) {
            assert_eq!(decoded[i], vec2[i]); // OJO
        }
    }

    #[test]
    fn test03_empty_array_encoding() {
        let vec: Vec<String> = vec![];
        let encoded = RArray::encode(vec);
        assert_eq!(encoded, "*0\r\n".to_string());
    }

    #[test]
    fn test04_empty_array_decoding() {
        let vec1: Vec<String> = vec![];
        let mut encoded: String = RArray::encode(vec1);
        encoded.remove(0); // Saco el '*'
        let decoded = RArray::decode(&mut encoded);
        assert_eq!(
            decoded.unwrap_err().print_it(),
            "ERR_EMPTY (empty array)".to_string()
        );
    }

    #[test]
    fn test05_set_key_value_simulation() {
        let input: String = "SET ping pong".to_string();
        let v: Vec<String> = input.split(' ').map(str::to_string).collect();
        let mut encoded: String = RArray::encode(v);

        assert_eq!(
            encoded,
            "*3\r\n$3\r\nSET\r\n$4\r\nping\r\n$4\r\npong\r\n".to_string()
        );

        encoded.remove(0);
        let mut decoded = RArray::decode(&mut encoded).unwrap();

        assert_eq!(decoded.remove(0), "SET".to_string());
        assert_eq!(decoded.remove(0), "ping".to_string());
        assert_eq!(decoded.remove(0), "pong".to_string());

        assert!(encoded.is_empty());
    }
}
