#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub runtimes: Vec<String>,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
	let mut s = config::Config::default();

	s.set("runtimes", vec![
	    "conmon".to_string(),
	    "containerd-shim".into()
	])?;

	s.merge(config::File::with_name("/etc/enclave/enclave.toml").
		required(false))?;
	s.try_into()
    }
}
