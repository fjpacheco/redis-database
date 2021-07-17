use std::io::{BufRead, Lines};

use super::{error::ErrorStruct, RArray};
use crate::{messages::redis_messages, native_types::bulk_string::RBulkString};

pub trait RedisType<T> {
    fn encode(t: T) -> String;
    fn decode<G>(
        first_lecture: String,
        redis_encoded_line: &mut Lines<G>,
    ) -> Result<T, ErrorStruct>
    where
        G: BufRead;
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

#[allow(dead_code)] //

pub fn verify_parsable_array_size<G>(
    sliced_size: String,
    rest: &mut Lines<G>,
) -> Result<Vec<String>, ErrorStruct>
where
    G: BufRead,
{
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
                "Failed to parse Redis array (1)".to_string(),
            ))
        } else {
            get_bulk_string_vector(size, rest)
        }
    } else {
        Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse Redis array (2)".to_string(),
        ))
    }
}

#[allow(dead_code)]
pub fn get_bulk_string_vector<G>(
    size: isize,
    rest: &mut Lines<G>,
) -> Result<Vec<String>, ErrorStruct>
where
    G: BufRead,
{
    let mut decoded_vec: Vec<String> = Vec::new();
    for _ in 0..size {
        fill_bulk_string_vector(rest, &mut decoded_vec)?;
    }
    Ok(decoded_vec)
}

#[allow(dead_code)]
pub fn fill_bulk_string_vector<G>(
    rest: &mut Lines<G>,
    decoded_vec: &mut Vec<String>,
) -> Result<(), ErrorStruct>
where
    G: BufRead,
{
    let mut first_lecture = rest.next().unwrap().unwrap();
    match first_lecture.remove(0) {
        // Redis Type inference
        '$' => {
            let array_elem = RBulkString::decode(first_lecture, rest)?;
            decoded_vec.push(array_elem);
            Ok(())
        }
        _ => Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse Redis array (3)".to_string(),
        )),
    }
}

#[allow(dead_code)]
pub fn verify_parsable_bulk_size<G>(
    sliced_size: String,
    rest_of: &mut Lines<G>,
) -> Result<String, ErrorStruct>
where
    G: BufRead,
{
    if let Ok(size) = sliced_size.parse::<isize>() {
        if size == -1 {
            Ok("(nil)".to_string())
        } else if size < 0 {
            Err(ErrorStruct::new(
                "ERR_PARSE".to_string(),
                "Failed to parse redis bulk string".to_string(),
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
fn split_b_string<G>(size: isize, rest_of: &mut Lines<G>) -> Result<String, ErrorStruct>
where
    G: BufRead,
{
    match rest_of.next() {
        Some(item) => verify_b_string_size(size, item.unwrap()),
        None => Err(ErrorStruct::new(
            "ERR_PARSE".to_string(),
            "Failed to parse redis bulk string".to_string(),
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
            "Failed to parse redis bulk string".to_string(),
        ))
    }
}

/// With **Netcat** you can received, for example a input: "set key value\r\n"
///
/// This function convert (by Redis Protocol),
///
/// "set key value\r\n"
///
/// in
///
/// "*3\r\n$3\r\nset\r\n$3\r\nkey\r\n$5\r\nvalue\r\n"
pub fn encode_netcat_input(line: String) -> Result<String, ErrorStruct> {
    let vector_words = line
        .split(' ')
        .collect::<Vec<&str>>()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let max_chars_in_command = 30;
    if vector_words
        .get(0)
        .unwrap_or(&"Empty".to_string())
        .len()
        .ge(&max_chars_in_command)
    {
        return Err(ErrorStruct::from(redis_messages::maximum_amount_exceeded(
            max_chars_in_command,
        )));
    }

    Ok(RArray::encode(vector_words))
}
