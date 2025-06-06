const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 5665;

pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {        
        let host = match std::env::var("MA_HOST") {
            Ok(h) => h,
            Err(_) => DEFAULT_HOST.to_string()
        };

        let port: u16 = match std::env::var("MA_PORT") {
            Ok(p) => p.parse().expect("MA_PORT must be of type u16"),
            Err(_) => DEFAULT_PORT
        };

        Config {
            host,
            port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn should_create_config_with_defaults() {
        let c = Config::from_env();

        assert_eq!(c.host, "127.0.0.1".to_string());
        assert_eq!(c.port, 5665);
    }

}

