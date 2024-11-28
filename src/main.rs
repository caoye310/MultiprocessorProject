// use std::sync::Arc; // 使用原子操作更新其引用计数，因此是线程安全的，可以跨线程共享。
// use std::env;
// use signal_hook::{iterator::Signals, consts::signal::SIGSEGV};
// use std::process;
// use std::thread;

use run_test::GlobalTest;
mod run_test;
mod link_list;
// Import your specific modules here:
// use rideables::{...};


// fn fault_handler() {
//     eprintln!("Segmentation fault detected.");
//     process::exit(1);
// }

fn main() {
    // let mut gtc = GlobalTestConfig::new();
    //
    // // Add Rideable options
    // gtc.add_rideable_option(Arc::new(LinkList::new()), "LinkList");
    // gtc.add_rideable_option(Arc::new(LinkListRange::new()), "LinkListRange");
    //
    // // Add Test options
    // gtc.add_test_option(Arc::new(ObjRetireTest::new(50, 0, 0, 30, 20, 65536, 5000)), "ObjRetire:g50i30rm20:range=65536:prefill=5000");
    // gtc.add_test_option(Arc::new(ObjRetireTest::new(50, 0, 50, 0, 0, 65536, 1024)), "ObjRetire:g50p50:range=65536:prefill=1024");

    // Parse command line


    // Run the test
    let test = GlobalTest::new(3,8);
    test.run_test();

    // Print results
    // if gtc.verbose {
    //     println!("Operations/sec: {}", gtc.total_operations / gtc.interval);
    // } else {
    //     print!("{}\t", gtc.total_operations / gtc.interval);
    // }
}

