use std::sync::OnceLock;

#[allow(dead_code)]
struct Config {
    env: String,
    is_local: bool,
    is_prd: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let env = std::env::var("COT_ENVIRONMENT").unwrap_or_else(|_| "localhost".to_string());
        let is_local = env == "localhost";
        let is_prd = env == "production";
        Config { env, is_local, is_prd}
    })
}

pub fn is_localhost() -> bool {
    get_config().is_local
}
#[allow(dead_code)]
pub fn environment() -> String {
    get_config().env.clone()
}