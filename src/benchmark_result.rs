use sqlx::{types::Decimal, Pool, Postgres};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};
use time::OffsetDateTime;

use crate::benchmarker::Benchmarker;

pub struct BenchmarkResult {
    plan: String,
    time: Decimal,
    path: Option<String>,
}

impl BenchmarkResult {
    pub async fn from_benchmarker(
        benchmarker: &Benchmarker,
        pool: &Pool<Postgres>,
    ) -> Result<Self, sqlx::Error> {
        let plan = benchmarker.get_plan(pool).await?;
        let time = benchmarker.benchmark(pool, None).await?;

        Ok(Self {
            plan,
            time,
            path: None,
        })
    }

    pub fn write(&mut self, path_prefix: impl AsRef<Path>) -> io::Result<()> {
        let result_path = format!("{}_result.txt", path_prefix.as_ref().to_string_lossy());
        let mut result_file = File::create(&result_path)?;

        writeln!(result_file, "Time: {}\n", OffsetDateTime::now_utc())?;
        writeln!(result_file, "Query plan:\n{}", self.plan)?;
        writeln!(result_file, "Benchmark: {}ms", self.time)?;

        result_file.flush()?;

        self.path = Some(result_path);

        Ok(())
    }

    pub fn report(&self) -> bool {
        if let Some(path) = &self.path {
            println!("Benchmark result stored in \"{}\"", path);

            true
        } else {
            false
        }
    }
}
