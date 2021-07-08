const TASK_COMM_LEN: usize = 16;

#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub runtimes: Vec<String>,
}

fn trim_task_comm_len(mut s: std::string::String) -> std::string::String {
    s.truncate(TASK_COMM_LEN - 1);
    s
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut s = config::Config::default();

        s.set("runtimes", vec![trim_task_comm_len("runc".to_string())])?;

        s.merge(config::File::with_name("/etc/enclave/enclave.toml").required(false))?;
        s.try_into()
    }
}
