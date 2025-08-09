#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub db_url: String,
    pub addr: String,
}

impl Config {
    #[inline]
    pub async fn parse() -> anyhow::Result<Self> {
        if let Err(error) = dotenvy::dotenv() {
            tracing::warn!("Failed to load .env: {error:?}");
        }

        let db_url = dotenvy::var("DATABASE_URL")?;
        let addr = dotenvy::var("ADDR")?;

        Ok(Self { addr, db_url })
    }
}
