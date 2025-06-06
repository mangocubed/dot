use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

pub(crate) static SESSION_CONFIG: LazyLock<SessionConfig> = LazyLock::new(|| extract_from_env("SESSION_"));

pub fn load_config() {
    use env_logger::{Env, init_from_env};

    let _ = dotenvy::dotenv();

    init_from_env(Env::new().default_filter_or("info"));
}

pub fn extract_from_env<'a, T>(prefix: &str) -> T
where
    T: Deserialize<'a> + Serialize + Default,
{
    use figment::Figment;
    use figment::providers::{Env, Serialized};

    Figment::from(Serialized::defaults(T::default()))
        .merge(Env::prefixed(prefix))
        .extract()
        .unwrap()
}

#[derive(Deserialize, Serialize)]
pub(crate) struct SessionConfig {
    pub domain: String,
    pub key: String,
    pub name: String,
    pub redis_url: String,
    pub secure: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            domain: "".to_owned(),
            key: "abcdefghijklmnopqrestuvvwxyz0123456789ABCDEFGHIJKLMNOPQRESTUVVWX".to_owned(),
            name: "_session".to_owned(),
            redis_url: "redis://127.0.0.1:6379/0".to_owned(),
            secure: false,
        }
    }
}
