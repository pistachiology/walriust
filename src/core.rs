use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{r2d2, PgConnection};
use std::env::VarError;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub telegram_token: String,
}

type ConnectionPool = Pool<ConnectionManager<PgConnection>>;


pub struct Kernel {
    pub db: ConnectionPool,

    config: Box<Config>,
}

impl Kernel {
    pub fn new(config: Box<Config>, db: ConnectionPool) -> Kernel {
        Kernel { config, db }
    }

    pub fn conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, r2d2::PoolError> {
        self.db.get()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl Clone for Kernel {
    fn clone(&self) -> Self {
        Kernel {
            config: self.config.clone(),
            db: self.db.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        let cloned = source.clone();
        self.config = cloned.config;
        self.db = cloned.db;
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key)
        .or_else(|_err| -> Result<String, VarError> { Ok(default.to_string()) })
        .unwrap()
}

fn build_config() -> Box<Config> {
    Box::new(Config {
        database_url: env_or_default("DATABASE_URL", "postgres:///walriust"),
        telegram_token: env_or_default("TELEGRAM_TOKEN", ""),
    })
}

fn build_db(config: &Config) -> Pool<ConnectionManager<PgConnection>> {
    let mgr = r2d2::ConnectionManager::<PgConnection>::new(&config.database_url);
    let builder = r2d2::Builder::new();
    let pool = builder
        .build(mgr)
        .expect("failed to create connection pool.");
    pool
}

pub fn build_kernel() -> Kernel {
    let config = build_config();
    let db = build_db(&config);

    Kernel::new(config, db)
}
