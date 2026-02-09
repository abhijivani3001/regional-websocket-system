use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub region: RegionConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub presence_ttl: u64, // TTL in seconds for presence information
    pub channel_prefix: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionConfig {
    pub name: String, // e.g., "us-east", "eu-west", "ap-south"
    pub other_regions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub messages_per_second: u32,
    pub burst_size: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenv::dotenv().ok();

        let settings = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.max_connections", 10000)?
            .set_default("redis.url", "redis://127.0.0.1:6379")?
            .set_default("redis.presence_ttl", 300)?
            .set_default("redis.channel_prefix", "ws_region")?
            .set_default("region.name", "us-east")?
            .set_default("region.other_regions", Vec::<String>::new())?
            .set_default("rate_limit.messages_per_second", 10)?
            .set_default("rate_limit.burst_size", 20)?
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?;

        settings.try_deserialize()
    }
}
