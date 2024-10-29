use dotenvy::dotenv;

#[derive(Debug)]
pub struct Config {
    pub app: Application,
}

impl Config {
    fn from_env() -> Self {
        let _ = dotenv();
        Config {
            app: Application::from_env(),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::from_env();
}

#[derive(Debug)]
pub struct Application {
    pub host: String,
    pub port: String,
    pub log_level: String,
}

impl Application {
    fn from_env() -> Self {
        Application {
            host: load_env("APP_HOST"),
            port: load_env("APP_PORT"),
            log_level: load_env("RUST_LOG"),
        }
    }

    pub fn server_url(&self) -> String {
        let Application { host, port, .. } = self;

        format!("{host}:{port}")
    }
}

fn load_env<T: std::convert::From<String>>(key: &str) -> T {
    std::env::var(key).map_or_else(
        |err| panic!("Failed to load {key} from env ({err})"),
        From::from,
    )
}
