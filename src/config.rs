use config::ConfigError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FilePath {
    pub orders: String,
    pub clients: String,
}

// Reading paths to Clients.txt and Orders.txt file
pub fn get_config() -> Result<FilePath, ConfigError> {
    let mut path = config::Config::default();
    path.merge(config::File::with_name("config"))?;
    path.try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config() {
        let config = get_config().unwrap();

        assert_eq!(config.orders, "./Orders.txt");
        assert_eq!(config.clients, "./Clients.txt");
    }
}
