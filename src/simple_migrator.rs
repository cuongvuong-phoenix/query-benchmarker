use anyhow::Result;
use sqlx::{migrate::Migrate, pool::PoolConnection, Connection, Executor, Pool, Postgres};
use std::borrow::Cow;
use time::Instant;

pub struct SimpleMigrator<'a> {
    path: Cow<'a, str>,
    sql: Cow<'a, str>,
    elapsed: Option<i128>,
}

impl<'a> SimpleMigrator<'a> {
    pub fn new(path: Cow<'a, str>, sql: Cow<'a, str>) -> Self {
        Self {
            path,
            sql,
            elapsed: None,
        }
    }

    pub async fn run(&mut self, pool: &Pool<Postgres>) -> Result<()> {
        let mut connection = pool.acquire().await?;

        // Lock the database for exclusive access by the migrator.
        connection.lock().await?;

        self.apply(&mut connection).await?;

        connection.unlock().await?;

        Ok(())
    }

    async fn apply(&mut self, connection: &mut PoolConnection<Postgres>) -> Result<()> {
        let start = Instant::now();
        let mut tx = connection.begin().await?;

        tx.execute(&*self.sql).await?;

        tx.commit().await?;
        self.elapsed = Some(start.elapsed().whole_milliseconds());

        Ok(())
    }

    pub fn report(&self) -> bool {
        if let Some(elapsed) = self.elapsed {
            println!("Migration ran for \"{}\" ({}ms)", self.path, elapsed);

            true
        } else {
            false
        }
    }
}
