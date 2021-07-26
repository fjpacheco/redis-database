///Methods with the format in String for the [InfoSv] command about a database.
pub mod info_db_formatter {
    pub fn title() -> String {
        "DATABASE:".to_string()
    }

    pub fn number_of_keys(n: usize) -> String {
        format!("Number of keys: {}", n)
    }
}

///Methods with the format in String for the [InfoSv] command about a clients and channels.
pub mod info_client_formatter {
    pub fn clients_connected(n: usize) -> String {
        format!("clients connected: {}", n)
    }

    pub fn active_channels(n: usize) -> String {
        format!("active channels: {}", n)
    }
}

///Methods with the format in String for the [InfoSv] command about a server.
pub mod info_server_formatter {
    use std::env::current_exe;

    pub fn title() -> String {
        "SERVER:".to_string()
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
