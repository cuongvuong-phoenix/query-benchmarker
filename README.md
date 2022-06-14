# Query benchmarker

A simple CLI tool to benchmark your queries.

## Prerequisities

- [Install *Rust*](https://www.rust-lang.org/learn/get-started).

- Install `sqlx-cli`:

  ```sh
  cargo install sqlx-cli
  ```

- Copy `.env.sample` into `.env` and edit `DATABASE_URL` variable:

  ```sh
  cp .env.sample .env
  ```

- Run base migrations:

  ```sh
  sqlx migrate run
  ```

## Basic Usage

- Create (if not exists) `sql` folder:

  ```sh
  mkdir sql
  ```

- Create a new folder inside `sql` (we call it a *section*) and some SQL scripts inside of it:

  ```sh
  mkdir sql/single_column_indexes
  touch sql/up.sql # Will be ran before section benchmarking.
  touch sql/down.sql # Will be ran after section benchmarking.
  touch sql/query.sql # The main query script (only 1 query accepted).
  ```

- Run the application and see the outputs:

  ```sh
  cargo run
  ```

## Structure

In general, the structure is as follows:

```tree
|-- sql
    |-- section_a
        -- up.sql
        -- down.sql
        -- query.sql # Override the shared `query.sql` file.
    |-- section_b
        up.sql
        down.sql
    -- query.sql
    -- shared-query.sql
|-- results
    |-- section_a
        -- <timestamp>_query.txt
        -- <timestamp>_shared-query.txt
        ...
    |-- section_b
        -- <timestamp>_query.txt
        -- <timestamp>_shared-query.txt
        ...
```

> **|--**: directory.
> **--**: file.
> **...**: more entries with the same pattern.
