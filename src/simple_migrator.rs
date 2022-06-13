use std::borrow::Cow;

use sqlx::{migrate::Migrate, pool::PoolConnection, Connection, Executor, Pool, Postgres};
use time::{Duration, Instant};

pub struct SimpleMigrator {
    path: Cow<'static, str>,
    sql: Cow<'static, str>,
    elapsed: Option<Duration>,
}

impl SimpleMigrator {
    pub fn new(path: Cow<'static, str>, sql: Cow<'static, str>) -> Self {
        Self {
            path,
            sql,
            elapsed: None,
        }
    }

    pub async fn run(&mut self, pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
        let mut connection = pool.acquire().await?;

        // Lock the database for exclusive access by the migrator.
        connection.lock().await?;

        self.apply(&mut connection).await?;

        connection.unlock().await?;

        Ok(())
    }

    async fn apply(
        &mut self,
        connection: &mut PoolConnection<Postgres>,
    ) -> Result<(), sqlx::Error> {
        let start = Instant::now();
        let mut tx = connection.begin().await?;

        tx.execute(&*self.sql).await?;

        tx.commit().await?;
        self.elapsed = Some(start.elapsed());

        Ok(())
    }

    pub fn report(&self) -> bool {
        if let Some(elapsed) = self.elapsed {
            println!("Migration ran for \"{}\" ({})", self.path, elapsed);

            true
        } else {
            false
        }
    }
}
