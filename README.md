### Evaluation of safe memory reclamation schemes in Rust

#### Usage of Hyaline
1. Get the throughput using Cargo:
```
cargo run -- 8 0.8
```
- The first argument is the number of threads.
- The second argument is the percentage of read operations (in decimal format).

Example Output:
```
Number of Threads: 8
Percentage of reading: 0.8
Execution time: 42560526500 nanosecond
```
Additionally, a file named memory_data.csv will be created in the current directory, containing the available memory data.

2. Run the built executable directly:
You can run the executable:
```
./target/debug/project 8 0.8
```
This will produce the same output as the cargo run command.

3. Generate the Memory Usage Plot:
Run the following command to create a memory usage plot:
```
python draw.py
```
- This command processes the data and generates a file named memory_usage_plot.png in the current directory.
- Ensure that the necessary dependencies (e.g., matplotlib) are installed before running this script.


https://drive.google.com/file/d/1srlIrPQjlsjKrhFGIdZ4FCKa0_wT8NPo/view?usp=sharing
