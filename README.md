### Evaluation of safe memory reclamation schemes in Rust

#### Usage
1. Get the throughput:
```
cargo run -- 8 0.8
```
The first argument is the number of threads and the second argument is the percentage of reading operations.

Output:
```
Number of Threads: 8
Percentage of reading: 0.8
Execution time: 42560526500 nanosecond
```
A file named memory_data.csv will be created. 
2. Draw
```
python draw.py
```
A file named memory_usage_plot will be created.