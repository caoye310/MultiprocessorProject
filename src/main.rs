use std::env;
use run_test::GlobalTest;
mod run_test;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        println!("Number of Threads: {}", args[1]);
        println!("Percentage of reading: {}", args[2]);
    } else {
        println!("Please enter the number of threads and percentage of reading operations！");
    }
    // Run the test
    let num_threads:i32 = args[1].parse().expect("Number of Threads!");
    let percentage:f64 = args[2].parse().expect("Percentage of Reading Operations!");
    let test = GlobalTest::new(32, num_threads, percentage);
    test.run_test();
}
