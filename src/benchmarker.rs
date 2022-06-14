use anyhow::{Error, Result};
use sqlx::{types::Decimal, Pool, Postgres};

pub struct Benchmarker {
    query: String,
}

impl Benchmarker {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
        }
    }

    pub async fn get_plan(&self, pool: &Pool<Postgres>) -> Result<String> {
        let query = format!("EXPLAIN ANALYZE {}", self.query);

        sqlx::query_as::<_, (String,)>(&query)
            .fetch_all(pool)
            .await
            .map(|records| {
                records.into_iter().fold("".to_string(), |accum, record| {
                    format!("{}{}\n", accum, record.0)
                })
            })
            .map_err(Error::msg)
    }

    pub async fn benchmark(&self, pool: &Pool<Postgres>, loop_times: i32) -> Result<Decimal> {
        let query = format!("SELECT benchmark($${}$$, {})", self.query, loop_times);
        sqlx::query_as::<_, (Decimal,)>(&query)
            .fetch_one(pool)
            .await
            .map(|record| record.0)
            .map_err(Error::msg)
    }
}
