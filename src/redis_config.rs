use crate::commands::server::info_formatter::info_server_formatter::*;
use crate::native_types::ErrorStruct;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, LineWriter},
    path::Path,
};

pub struct RedisConfig {
    ip: String,
    port: String,
    log_filename: String,
    lwriter: LineWriter<File>,
    _dump_filename: String,
    dump_file: File,
    verbose: usize,
    timeout_secs: u64,
}

impl RedisConfig {
    pub fn new(
        ip: String,
        port: String,
        log_filename: String,
        _dump_filename: String,
        verbose: usize,
    ) -> Result<RedisConfig, ErrorStruct> {
        let lwriter;
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log_filename)
        {
            Ok(file) => {
                lwriter = LineWriter::new(file);
            }
            Err(err) => {
                return Err(ErrorStruct::new(
                    "ERR_CONFIG".into(),
                    format!("Setting a new config failed. Detail: {}", err),
                ))
            }
        }

        let dump_file = match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&_dump_filename)
        {
            Ok(file) => file,
            Err(err) => {
                return Err(ErrorStruct::new(
                    "ERR_CONFIG".into(),
                    format!("Setting a new config failed. Detail: {}", err),
                ))
            }
        };
        Ok(RedisConfig {
            ip,
            port,
            log_filename,
            verbose,
            lwriter,
            _dump_filename,
            dump_file,
            timeout_secs: 0,
        })
    }

    pub fn info(&self, info_compiler: &mut Vec<String>) {
        info_compiler.push(title());
        info_compiler.push(ip(&self.ip));
        info_compiler.push(port(&self.port));
        info_compiler.push(current_exe_dir());
        info_compiler.push(logfile_name(&self.log_filename));
        info_compiler.push(verbose_level(self.verbose));
        info_compiler.push(client_timeout(self.timeout_secs));
        info_compiler.push(String::new());
    }

    pub fn parse_config(argv: Vec<String>) -> Result<RedisConfig, ErrorStruct> {
        let config = match argv.len().eq(&2) {
            true => RedisConfig::get_with_new_config(&argv[1])?,
            false => RedisConfig::default(),
        };
        Ok(config)
    }

    pub fn get_mut_linewriter(&mut self) -> Option<&mut LineWriter<File>> {
        Some(&mut self.lwriter)
    }

    pub fn get_mut_dump_file(&mut self) -> Option<&mut File> {
        Some(&mut self.dump_file)
    }

    pub fn change_log_file(&mut self, new_log_filename: String) -> Result<(), ErrorStruct> {
        match OpenOptions::new()
            .append(true)
            .create(true)
            .open(new_log_filename)
        {
            Ok(file) => {
                self.lwriter = LineWriter::new(file);
                Ok(())
            }
            Err(err) => Err(ErrorStruct::new(
                "ERR_CONFIG".into(),
                format!("Setting a new config failed. Detail: {}", err),
            )),
        }
    }

    pub fn change_dump_file(&mut self, new_dump_filename: String) -> Result<(), ErrorStruct> {
        self.dump_file = match OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&new_dump_filename)
        {
            Ok(file) => file,
            Err(err) => {
                return Err(ErrorStruct::new(
                    "ERR_CONFIG".into(),
                    format!("Setting a new config failed. Detail: {}", err),
                ))
            }
        };
        Ok(())
    }

    /// Received name path of new file .config to generate a new configuration for server Redis
    ///
    /// If the .config file comes incomplete, the default configurations will be taken.
    ///
    /// ## Error
    ///
    /// Return Err if ...
    fn get_with_new_config(path: &str) -> Result<RedisConfig, ErrorStruct> {
        let file = match File::open(Path::new(path)) {
            Err(err) => Err(ErrorStruct::new(
                "ERR_CONFIG".into(),
                format!("Set a new config failure. Detail: {}", err),
            )),
            Ok(file) => Ok(file),
        }?;

        let config = get_configs(file);
        let ip = "127.0.0.1".into();

        let port = config
            .get("port")
            .unwrap_or(&Self::default().port())
            .to_string();
        RedisConfig::new(
            ip,
            port,
            String::from("logs.txt"),
            "dump.rdb".to_string(),
            0,
        )
    }

    pub fn ip(&self) -> String {
        self.ip.to_string()
    }

    pub fn port(&self) -> String {
        self.port.to_string()
    }

    pub fn timeout(&self) -> u64 {
        self.timeout_secs
    }

    pub fn get_addr(&self) -> String {
        self.ip.to_string() + ":" + &self.port
    }

    pub fn update_port(&mut self, port: &str) {
        self.port = port.to_string();
    }

    pub fn db_filename(&self) -> String {
        "TODO".to_string()
    }

    pub fn log_filename(&self) -> String {
        String::from(&self.log_filename)
    }

    pub fn verbose(&self) -> &usize {
        &self.verbose
    }

    pub fn change_verbose(&mut self, new: usize) {
        self.verbose = new;
    }
}

impl Default for RedisConfig {
    /// Obtained a default configuration for server Redis
    ///
    /// ## Redis Config by Default:
    ///
    /// * **IP**: 127.0.0.1
    ///
    /// * **PORT**: 6379
    fn default() -> Self {
        let ip = "127.0.0.1".into();
        let port = "6379".into();
        let log_filename = "logs.txt".to_string();
        let verbose = 100;
        RedisConfig::new(ip, port, log_filename, "dump.rdb".to_string(), verbose).unwrap()
    }
}

/// Each line of the [File] is inserted into a [HashMap] with format:
///
/// * **key** => *"name_config"*
///
/// * **value** => *"estate_config"*
///
/// ## Example
///
/// A line from the file with "port 7777" in the HashMap was saved (key, value): (port, 7777)
fn get_configs(file: File) -> HashMap<String, String> {
    BufReader::new(file)
        .lines()
        .into_iter()
        .map(|x| x.unwrap_or_else(|_| " ".to_string()))
        .filter(|x| !x.starts_with('#'))
        .filter(|x| !x.is_empty())
        .map(|x| {
            x.split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>()
        })
        .map(|x| {
            (
                x.get(0).unwrap().to_string(), // No empty! Unwrap its ok
                x.get(1).unwrap_or(&" ".to_string()).to_string(),
            )
        })
        .collect::<HashMap<String, String>>() // Functional Rust :')
}
