use std::fs;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use env_logger::Env;
use sqlx::prelude::FromRow;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Executor, SqlitePool};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    cleanup();
    let conn_options = SqliteConnectOptions::from_str("sqlite://test.db")?
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5))
        .synchronous(SqliteSynchronous::Normal)
        .statement_cache_capacity(1_000_000_000)
        .foreign_keys(true)
        .pragma("txlock", "immediate")
        .pragma("tempstore", "memory");

    let write_options = conn_options.clone().create_if_missing(true);
    let write_pool_options = SqlitePoolOptions::new().max_connections(1);
    let write_db = write_pool_options.connect_with(write_options).await?;

    let read_options = conn_options.read_only(true);
    let read_pool_options = SqlitePoolOptions::new()
        .max_connections(num_cpus::get_physical().max(4).try_into().unwrap());
    let read_db = read_pool_options.connect_with(read_options).await?;

    sqlx::query(
        "CREATE TABLE test (
        id BLOB NOT NULL PRIMARY KEY,
        timestamp INTEGER NOT NULL,
        counter INT NOT NULL
    ) STRICT",
    )
    .execute(&write_db)
    .await?;

    log::info!("Inserting 5,000,000 rows");
    setup_db(&write_db).await?;

    let record_id_to_find: Uuid =
        sqlx::query_scalar("SELECT id FROM test ORDER BY id DESC LIMIT 1")
            .fetch_one(&read_db)
            .await?;

    log::info!("Starting benchmark");

    let concurrent_readers = 500;
    let concurrent_writers = 1;

    let reads = Arc::new(AtomicI64::new(0));
    let writes = Arc::new(AtomicI64::new(0));

    let start = Instant::now();

    let mut join_handles = Vec::new();
    for _c in 0..concurrent_readers {
        let reads_t = reads.clone();
        join_handles.push(tokio::spawn({
            let record_id_to_find_local = record_id_to_find;
            let read_db_local = read_db.clone();
            async move {
                let mut reads_local = 0;

                let _ = tokio::time::timeout(Duration::from_secs(10), async {
                    loop {
                        let _row: Option<TestRecord> =
                            match sqlx::query_as("SELECT * FROM test WHERE id = ?")
                                .bind(record_id_to_find_local.as_bytes().as_slice())
                                .fetch_optional(&read_db_local)
                                .await
                            {
                                Err(err) => {
                                    log::error!("Error: {:#?}", err);
                                    break;
                                }
                                Ok(val) => {
                                    if val.is_none() {
                                        log::error!("Got None");
                                    }
                                    val
                                }
                            };
                        reads_local += 1;
                        // tokio::task::yield_now().await;
                    }
                })
                .await;

                reads_t.fetch_add(reads_local, Ordering::Relaxed);
            }
        }));
    }

    for _c in 0..concurrent_writers {
        join_handles.push(tokio::spawn({
            let write_db = write_db.clone();
            let writes_t = writes.clone();
            async move {
                let timestamp = Utc::now().timestamp_millis();
                let mut writes_local = 0;
                let _ = tokio::time::timeout(Duration::from_secs(10), async {
                    loop {
                        let record_id = Uuid::now_v7();
                        if let Err(err) =
                            sqlx::query("INSERT INTO test (id, timestamp, counter) VALUES (?,?,?)")
                                .bind(record_id.as_bytes().as_slice())
                                .bind(timestamp)
                                .bind(writes_local)
                                .execute(&write_db)
                                .await
                        {
                            log::error!("Error: {:#?}", err);
                            break;
                        };
                        writes_local += 1;
                    }
                })
                .await;

                writes_t.fetch_add(writes_local, Ordering::Relaxed);
            }
        }));
    }

    for jh in join_handles {
        jh.await?;
    }

    let elapsed = start.elapsed();

    log::info!("Benchmark stopped: {:?}", elapsed);
    println!("------------------------");

    let reads = reads.load(Ordering::Relaxed);
    log::info!("{} reads", reads);

    let throughput_read = reads as f64 / elapsed.as_secs_f64();
    log::info!("{} reads/s", throughput_read);

    println!("------------------------");

    let writes = writes.load(Ordering::Relaxed);
    log::info!("{} writes", writes);

    let throughput_write = writes as f64 / elapsed.as_secs_f64();
    log::info!("{} writes/s", throughput_write);
    Ok(())
}

fn cleanup() {
    let _ = fs::remove_file("./test.db");
    let _ = fs::remove_file("./test.db-shm");
    let _ = fs::remove_file("./test.db-wal");
}
async fn setup_db(db: &SqlitePool) -> eyre::Result<()> {
    let mut tx = db.begin().await?;
    // let start = Instant::now();

    let timestamp = Utc::now().timestamp_millis();
    for i in 0..5_000_000 {
        let record_id = Uuid::now_v7();
        sqlx::query("INSERT INTO test (id, timestamp, counter) VALUES (?, ?, ?)")
            .bind(record_id.as_bytes().as_slice())
            .bind(timestamp)
            .bind(i)
            //.execute(db)
            .execute(&mut *tx)
            .await?;

        // insert by batches of 500,000 rows
        if i % 500_000 == 0 {
            tx.commit().await?;
            // println!("Batch completed: {:?}", start.elapsed());
            tx = db.begin().await?;
        }
    }
    tx.commit().await?;
    // println!("Last batch completed: {:?}", start.elapsed());

    db.execute("VACUUM").await?;
    db.execute("ANALYZE").await?;
    // println!("Setup completed: {:?}", start.elapsed());
    Ok(())
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
struct TestRecord {
    id: Uuid,
    timestamp: i64,
    counter: i64,
}
