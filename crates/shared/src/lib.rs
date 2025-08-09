pub mod config;
pub mod db;
pub mod error;
pub mod domain {
    pub mod event;
}
pub mod contracts;
pub mod repo;
pub mod seed;

pub use config::AppConfig;
