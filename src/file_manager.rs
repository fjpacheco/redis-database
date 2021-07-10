use crate::native_types::error::ErrorStruct;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Error, LineWriter, Write};
use std::path::Path;

use crate::native_types::{RBulkString, RedisType};

#[allow(dead_code)]
pub struct FileManager;

impl FileManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        FileManager {}
    }
    #[allow(dead_code)]
    pub fn open_file(filename: &str) -> Result<BufReader<File>, Error> {
        let path = Path::new(filename);
        let _display = path.display();
        let file = File::open(&path)?;

        Ok(BufReader::new(file))
    }

    #[allow(dead_code)]
    pub fn read_line(self, filename: &str) -> Result<String, ErrorStruct> {
        // READS ENCODED TEXT, DECODES IT AND RETURNS IT
        let encoded_text = fs::read_to_string(filename).unwrap();
        let mut bufreader = BufReader::new(encoded_text.as_bytes());
        let mut first_lecture = String::new();
        let _decoded = bufreader.read_line(&mut first_lecture);
        first_lecture.remove(0); // Redis Type inference
        first_lecture.pop().unwrap(); // popping \n
        first_lecture.pop().unwrap(); // popping \r
        RBulkString::decode(first_lecture, &mut bufreader.lines())
    }

    #[allow(dead_code)]
    pub fn write_to_file(
        &self,
        file: &mut LineWriter<File>,
        text: String,
    ) -> Result<(), ErrorStruct> {
        // RECEIVES TEXT, ENCODES IT AND WRITES IT
        let encoded_text = RBulkString::encode(text);
        if file.write_all(&encoded_text.into_bytes()).is_ok() {
            Ok(())
        } else {
            Err(ErrorStruct::new(
                "ERR_WRITE".to_string(),
                "File write failed".to_string(),
            ))
        }
        // if last line doesn't end in a newline flush or drop LineWriter is needed to finish writing
        // file.flush()?;
        // Ok(())
    }
}
impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test_file_manager {

    use super::FileManager;
    use std::{
        fs::{self, File},
        io::LineWriter,
    };

    #[test]
    fn test01_write_line() {
        let file_manager = FileManager::new();
        let text = "This is a line to be written".to_string();
        let file = File::create("example1.txt").unwrap();
        // file.set_len(0);
        let mut file = LineWriter::new(file);
        file_manager.write_to_file(&mut file, text).unwrap();

        assert_eq!(
            fs::read("example1.txt").unwrap(),
            b"$28\r\nThis is a line to be written\r\n"
        );
    }

    #[test]
    fn test02_write_many_lines() {
        let file_manager = FileManager::new();
        let file = File::create("example2.txt").unwrap();
        // file.set_len(0);
        let mut file = LineWriter::new(file);

        let text1 = "This is a line to be written".to_string();
        file_manager.write_to_file(&mut file, text1).unwrap();

        let text2 = "This is a line to be written".to_string();
        file_manager.write_to_file(&mut file, text2).unwrap();

        let text3 = "This is a line to be written".to_string();
        file_manager.write_to_file(&mut file, text3).unwrap();

        let should_be = b"$28\r\nThis is a line to be written\r\n$28\r\nThis is a line to be written\r\n$28\r\nThis is a line to be written\r\n";
        assert_eq!(fs::read("example2.txt").unwrap(), &should_be[..]);
    }

    #[test]
    fn test03_read_line() {
        let file_manager = FileManager::new();
        let text = "This is a line to be written".to_string();
        let file = File::create("example3.txt").unwrap();
        let mut file = LineWriter::new(file);
        file_manager.write_to_file(&mut file, text).unwrap();
        let read = file_manager.read_line("example3.txt").unwrap();
        assert_eq!(read, "This is a line to be written".to_string());
    }
}
