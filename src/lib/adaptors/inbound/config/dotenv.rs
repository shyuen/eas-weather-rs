use dotenv::dotenv;

use crate::core::ports::inbound::config::ConfigEnvFileRepo;

pub struct env_file;

impl ConfigEnvFileRepo for env_file {
    /// Loads environment variables from a .env file.
    fn set_env_from_file(&self) {
        dotenv().ok();
    }
}
