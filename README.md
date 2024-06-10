# optimizing-sqlite-for-servers-rust

- Based on <https://kerkour.com/sqlite-for-servers>
- Adapted to rust

## Example run sqlitebench (Go)

```bash
> go run main.go 
2024/06/10 11:45:29 Inserting 5,000,000 rows
2024/06/10 11:46:03 Starting benchmark
2024/06/10 11:46:13 Benchmark stopped: 10.002515085s
----------------------
2024/06/10 11:46:13 1736934 reads
2024/06/10 11:46:13 173649.725618 reads/s
----------------------
2024/06/10 11:46:13 54412 writes
2024/06/10 11:46:13 5439.831836 writes/s
```

## Example run sqlx

```sh
> cargo run -p sqlx-sqlite-opt --example quick_dev --release
   Compiling sqlx-sqlite-opt v0.1.0 (/home/kristoffer/projekt/kristoff/optimizing-sqlite-for-servers-rust/crates/sqlx-sqlite-opt)
    Finished `release` profile [optimized] target(s) in 3.43s
     Running `target/release/examples/quick_dev`
[2024-06-10T11:36:32Z INFO  quick_dev] Inserting 5,000,000 rows
[2024-06-10T11:37:38Z WARN  sqlx::query] slow statement: execution time exceeded alert threshold summary="VACUUM" db.statement="" rows_affected=1 rows_returned=0 elapsed=2.682693383s elapsed_secs=2.682693383 slow_threshold=1s
[2024-06-10T11:37:38Z INFO  quick_dev] Starting benchmark
[2024-06-10T11:37:48Z INFO  quick_dev] Benchmark stopped: 10.002049582s
------------------------
[2024-06-10T11:37:48Z INFO  quick_dev] 354050 reads
[2024-06-10T11:37:48Z INFO  quick_dev] 35397.74494191265 reads/s
------------------------
[2024-06-10T11:37:48Z INFO  quick_dev] 52194 writes
[2024-06-10T11:37:48Z INFO  quick_dev] 5218.330460381835 writes/s
```
