pub mod info_db_formatter {

    pub fn title() -> String {
        format!("DATABASE:")
    }

    pub fn number_of_keys(n: usize) -> String {
        format!("Number of keys: {}", n)
    }

}

pub mod info_client_formatter {

    pub fn clients_connected(n: usize) -> String {
        format!("clients connected: {}", n)
    }

    pub fn active_channels(n: usize) -> String {
        format!("active channels: {}", n)
    }

}

pub mod info_server_formatter {

    use std::env::current_exe;

    pub fn title() -> String {
        format!("SERVER:")
    }

    pub fn ip(ip: &str) -> String {
        format!("ip: {}", ip)
    }

    pub fn port(port: &str) -> String {
        format!("tcp port: {}", port)
    }

    pub fn current_exe_dir() -> String {
        format!("executable: {:?}", current_exe())
    }

    pub fn verbose_level(v: usize) -> String {
        format!("verbose level: {}", v)
    }

    pub fn client_timeout(t: u64) -> String {
        format!("connection timeout: {}", t)
    }

    pub fn logfile_name(name: &str) -> String {
        format!("log file name: {}", name)
    }

}