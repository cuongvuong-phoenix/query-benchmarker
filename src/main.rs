use dotenv::dotenv;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

mod benchmarker;

use crate::benchmarker::Benchmarker;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    // Database.
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in the `.env` file");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run temp migrations.
    let migrator = Migrator::new(Path::new("./migrations")).await?;

    migrator.run(&pool).await?;

    // Read files.
    for entry in fs::read_dir("sql")? {
        let outer_path = entry?.path();

        if outer_path.is_dir() {
            let outer_path_str = outer_path.to_string_lossy();

            let migration_paths = vec![
                format!("{}/up.sql", outer_path_str),
                format!("{}/down.sql", outer_path_str),
            ];
            // ----------------------------------------------------------------
            // Up runner
            // ----------------------------------------------------------------
            if let Ok(contents) = fs::read(&migration_paths[0]) {
                sqlx::query(&String::from_utf8_lossy(&contents))
                    .execute(&pool)
                    .await?;
            }

            // ----------------------------------------------------------------
            // Queries runner
            // ----------------------------------------------------------------
            for query_entry in fs::read_dir(&outer_path)? {
                let query_path = query_entry?.path();
                let query_path_str = query_path.to_string_lossy().to_string();

                if query_path.is_file()
                    && query_path.extension().unwrap() == "sql"
                    && !migration_paths.contains(&query_path_str)
                {
                    let contents = fs::read(&query_path)?;

                    let benchmarker = Benchmarker::new(&String::from_utf8_lossy(&contents));

                    let plan = benchmarker.get_plan(&pool).await?;
                    let time = benchmarker.benchmark(&pool, None).await?;

                    // Write results.
                    let query_file_name = query_path.with_extension("");
                    let result_path = format!("{}_result.txt", query_file_name.to_string_lossy());
                    let mut result = File::create(&result_path)?;

                    write!(result, "Query plan:\n{}\n", plan)?;
                    write!(result, "Benchmark: {}ms", time)?;

                    result.flush()?;

                    println!("Result stored in \"{}\"", result_path)
                }
            }

            // ----------------------------------------------------------------
            // Down runner
            // ----------------------------------------------------------------
            if let Ok(contents) = fs::read(&migration_paths[1]) {
                sqlx::query(&String::from_utf8_lossy(&contents))
                    .execute(&pool)
                    .await?;
            }
        }
    }

    // Revert temp migrations.
    migrator.undo(&pool, 1).await?;

    Ok(())
}
