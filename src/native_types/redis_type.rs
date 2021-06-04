use std::io::BufRead;

use super::{bulk_string::RBulkString, error::ErrorStruct};

pub trait RedisType<T> {
    fn encode(t: T) -> String;
    fn decode(redis_encoded_line: &mut dyn BufRead) -> Result<T, ErrorStruct>;
}

pub fn remove_first_cr_lf(slice: &mut String) -> Option<String> {
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

#[allow(dead_code)]
pub fn verify_parsable_array_size(
    sliced_size: String,
    rest: &mut dyn BufRead,
) -> Result<Vec<String>, ErrorStruct> {
    if let Ok(size) = sliced_size.parse::<isize>() {
        // *0\r\n
        if size == 0 {
            Err(ErrorStruct::new(
                "ERR_EMPTY".to_string(),
                "(empty array)".to_string(),
            ))
        } else if size < 1 {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse Redis array".to_string(),
            ))
        } else {
            // Analizar qué sucede si size = 0
            get_bulk_string_vector(size, rest)
        }
    } else {
        Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse Redis array".to_string(),
        ))
    }
}

#[allow(dead_code)]
fn get_bulk_string_vector(size: isize, rest: &mut dyn BufRead) -> Result<Vec<String>, ErrorStruct> {
    let mut decoded_vec: Vec<String> = Vec::new();
    for __ in 0..size {
        fill_bulk_string_vector(rest, &mut decoded_vec)?;
    }
    Ok(decoded_vec)
}

#[allow(dead_code)]
fn fill_bulk_string_vector(
    rest: &mut dyn BufRead,
    decoded_vec: &mut Vec<String>,
) -> Result<(), ErrorStruct> {
    let mut buf = vec![0; 1];
    match rest.read_exact(&mut buf) {
        Ok(_) => {
            let array_elem = RBulkString::decode(rest)?;
            decoded_vec.push(array_elem);
            Ok(())
        }
        Err(_) => Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse array".to_string(),
        )),
    }
}

#[allow(dead_code)]
pub fn verify_parsable_bulk_size(
    sliced_size: String,
    rest_of: &mut dyn BufRead,
) -> Result<String, ErrorStruct> {
    if let Ok(size) = sliced_size.parse::<isize>() {
        if size == -1 {
            Ok("(nil)".to_string())
        } else if size < 0 {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse Redis bulk string".to_string(),
            ))
        } else {
            // ¿Puede haber size = 0?
            split_b_string(size, rest_of)
        }
    } else {
        Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse redis bulk string".to_string(),
        ))
    }
}

#[allow(dead_code)]
fn split_b_string(size: isize, rest_of: &mut dyn BufRead) -> Result<String, ErrorStruct> {
    let mut sliced_b_string = String::new();
    match rest_of.read_line(&mut sliced_b_string) {
        Ok(_) => {
            sliced_b_string.pop();
            sliced_b_string.pop();
            verify_b_string_size(size, sliced_b_string)
        }
        Err(_) => Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse redis string".to_string(),
        )),
    }
}

#[allow(dead_code)]
fn verify_b_string_size(size: isize, sliced_b_string: String) -> Result<String, ErrorStruct> {
    if sliced_b_string.len() == size as usize {
        Ok(sliced_b_string)
    } else {
        Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse redis simple string".to_string(),
        ))
    }
}
