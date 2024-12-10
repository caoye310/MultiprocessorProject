use run_test::GlobalTest;
mod run_test;


fn main() {

    // Run the test
    for i in [0.8] {
        let test = GlobalTest::new(32, 8,i);
        test.run_test();
    }
}
