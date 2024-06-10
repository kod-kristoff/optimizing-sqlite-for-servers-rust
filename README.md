# optimizing-sqlite-for-servers-rust

- Based on <https://kerkour.com/sqlite-for-servers>
- Adapted to rust

## Example run sqlx

```sh
Inserting 5,000,000 rows
Batch completed: 510.266µs
Batch completed: 5.522293682s
Batch completed: 11.01568972s
Batch completed: 16.775657718s
Batch completed: 22.345852222s
Batch completed: 27.897444396s
Batch completed: 33.497580718s
Batch completed: 39.138281743s
Batch completed: 44.915136349s
Batch completed: 50.555347602s
Last batch completed: 56.128809518s
Setup completed: 59.008494188s
Starting benchmark
Benchmark stopped: 10.002143788s
------------------------
432965 reads
43287.22013769155 reads/s
------------------------
51034 writes
5102.306173725245 writes/s
```
