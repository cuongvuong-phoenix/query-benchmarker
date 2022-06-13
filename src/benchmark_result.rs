use anyhow::Result;
use sqlx::{types::Decimal, Pool, Postgres};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::benchmarker::Benchmarker;

pub struct BenchmarkResult {
    plan: String,
    time: Decimal,
    path: Option<PathBuf>,
}

impl BenchmarkResult {
    pub async fn from_benchmarker(
        benchmarker: &Benchmarker,
        pool: &Pool<Postgres>,
    ) -> Result<Self> {
        let plan = benchmarker.get_plan(pool).await?;
        let time = benchmarker.benchmark(pool, None).await?;

        Ok(Self {
            plan,
            time,
            path: None,
        })
    }

    pub fn write(
        &mut self,
        time_benchmark: &str,
        results_dir: impl AsRef<Path>,
        section_stem: &OsStr,
        query_stem: &OsStr,
    ) -> Result<()> {
        let section_result_dir = results_dir.as_ref().join(section_stem);

        if let Err(_) = fs::read_dir(&section_result_dir) {
            fs::create_dir(&section_result_dir)?;
        }

        let result_path = section_result_dir
            .join(format!(
                "{}_{}",
                time_benchmark,
                query_stem.to_string_lossy()
            ))
            .with_extension("txt");

        let mut result_file = File::create(&result_path)?;

        writeln!(result_file, "Benchmark: {} ms\n", self.time)?;
        writeln!(result_file, "Query plan:\n{}", self.plan)?;

        result_file.flush()?;

        self.path = Some(result_path);

        Ok(())
    }

    pub fn report(&self) -> bool {
        if let Some(path) = &self.path {
            println!("Benchmark result stored in \"{}\"", path.display());

            true
        } else {
            false
        }
    }
}
