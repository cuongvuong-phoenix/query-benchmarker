use anyhow::{Context, Result};
use dotenv::dotenv;
use simple_migrator::SimpleMigrator;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use time::{format_description, OffsetDateTime};

mod benchmark_result;
mod benchmarker;
mod simple_migrator;
mod utilities;

use benchmark_result::BenchmarkResult;
use benchmarker::Benchmarker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let time_benchmark = OffsetDateTime::now_utc().format(&format_description::parse(
        "[year][month][day]-[hour][minute][second]",
    )?)?;

    // Database.
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in the `.env` file");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run temp migrations.
    let migrator = Migrator::new(Path::new("./migrations")).await?;

    migrator.run(&pool).await?;

    // Temp.
    let migration_names = vec![String::from("up.sql"), String::from("down.sql")];

    let sql_path = Path::new("sql").canonicalize()?;

    // Store shared queries.
    let mut shared_query_paths = vec![];

    for shared_query_entry in fs::read_dir(&sql_path)? {
        let shared_query_path = shared_query_entry?.path();

        if shared_query_path.is_file() && utilities::is_sql_script(&shared_query_path) {
            shared_query_paths.push(shared_query_path);
        }
    }

    // Create `results` dir if not exists.
    let results_path = Path::new("results");

    if let Err(_) = fs::read_dir(&results_path) {
        fs::create_dir(&results_path)?;
    }

    let results_path = results_path.canonicalize()?;

    // Read files.
    for outer_entry in fs::read_dir("sql")? {
        let outer_path = outer_entry?.path();

        if outer_path.is_dir() {
            println!();

            // ----------------------------------------------------------------
            // Up runner
            // ----------------------------------------------------------------
            let up_path = outer_path.join(&migration_names[0]);

            if let Ok(contents) = fs::read_to_string(&up_path) {
                let mut migrator =
                    SimpleMigrator::new(up_path.to_string_lossy(), Cow::Owned(contents));

                migrator.run(&pool).await?;
                migrator.report();
            }

            // ----------------------------------------------------------------
            // Queries runner
            // ----------------------------------------------------------------
            let mut query_paths = shared_query_paths.clone();

            // Collect distinct queries.
            for query_entry in fs::read_dir(&outer_path)? {
                let query_path = query_entry?.path();
                let query_file_name = query_path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .with_context(|| format!("Path \"{}\" terminated", query_path.display()))?;

                if query_path.is_file()
                    && utilities::is_sql_script(&query_path)
                    && !migration_names.iter().any(|name| name == query_file_name)
                {
                    if let Some(index) = query_paths
                        .iter()
                        .map(|path| path.file_name().unwrap())
                        .position(|name| name == query_file_name)
                    {
                        query_paths[index] = query_path;
                    } else {
                        query_paths.push(query_path);
                    }
                }
            }

            // Run queries.
            for query_path in query_paths {
                let contents = fs::read_to_string(&query_path)?;
                let benchmarker = Benchmarker::new(&contents);
                let mut benchmark_result =
                    BenchmarkResult::from_benchmarker(&benchmarker, &pool).await?;

                if let Some(stem) = query_path.file_stem() {
                    benchmark_result.write(
                        &time_benchmark,
                        &results_path,
                        outer_path.file_stem().unwrap(),
                        stem,
                    )?;
                    benchmark_result.report();
                }
            }

            // ----------------------------------------------------------------
            // Down runner
            // ----------------------------------------------------------------
            let down_path = outer_path.join(&migration_names[1]);

            if let Ok(contents) = fs::read_to_string(&down_path) {
                let mut migrator =
                    SimpleMigrator::new(down_path.to_string_lossy(), Cow::Owned(contents));

                migrator.run(&pool).await?;
                migrator.report();
            }
        }
    }

    // Revert temp migrations.
    migrator.undo(&pool, 1).await?;

    Ok(())
}
