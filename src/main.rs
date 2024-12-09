use run_test::GlobalTest;
mod run_test;


fn main() {

    // Run the test
    let test = GlobalTest::new(8,8);
    test.run_test();
}

