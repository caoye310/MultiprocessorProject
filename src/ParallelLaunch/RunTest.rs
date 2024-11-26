use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

// Struct for storing global test configurations
struct GlobalTestConfig {
    rideable_factories: Vec<Box<dyn RideableFactory>>,
    rideable_names: Vec<String>,
    tests: Vec<Box<dyn Test>>,
    test_names: Vec<String>,
    out_file: Option<String>,
    allocated_rideables: Vec<Box<dyn Rideable>>,
    environment: HashMap<String, String>,
    arguments: HashMap<String, *mut std::ffi::c_void>,
    interval: usize,
    verbose: bool,
    warmup: usize,
    task_num: usize,
    task_stall: usize,
    test_type: usize,
    rideable_type: usize,
    affinity: String,
    recorder: Option<Recorder>,
}

impl GlobalTestConfig {
    fn new() -> Self {
        Self {
            rideable_factories: vec![],
            rideable_names: vec![],
            tests: vec![],
            test_names: vec![],
            out_file: None,
            allocated_rideables: vec![],
            environment: HashMap::new(),
            arguments: HashMap::new(),
            interval: 0,
            verbose: false,
            warmup: 0,
            task_num: 0,
            task_stall: 0,
            test_type: 0,
            rideable_type: 0,
            affinity: String::new(),
            recorder: None,
        }
    }

    fn alloc_rideable(&mut self) -> Box<dyn Rideable> {
        let rideable = self.rideable_factories[self.rideable_type]
            .build(self)
            .expect("Failed to build rideable");
        self.allocated_rideables.push(rideable.clone());
        rideable
    }

    fn print_arg_def(&self) {
        eprintln!("Usage: program_name [options]");
        for (i, name) in self.rideable_names.iter().enumerate() {
            eprintln!("Rideable {}: {}", i, name);
        }
        for (i, name) in self.test_names.iter().enumerate() {
            eprintln!("Test Mode {}: {}", i, name);
        }
    }

    fn parse_command_line(&mut self, args: Vec<String>) {
        let mut opts = clap::App::new("GlobalTestConfig")
            .arg(clap::Arg::with_name("interval").short("i").takes_value(true))
            .arg(clap::Arg::with_name("verbose").short("v"))
            .arg(clap::Arg::with_name("warmup").short("w").takes_value(true))
            .arg(clap::Arg::with_name("task_num").short("t").takes_value(true))
            .arg(clap::Arg::with_name("test_type").short("m").takes_value(true))
            .arg(clap::Arg::with_name("rideable_type").short("r").takes_value(true))
            .arg(clap::Arg::with_name("affinity").short("a").takes_value(true))
            .arg(clap::Arg::with_name("out_file").short("o").takes_value(true));

        let matches = opts.get_matches_from(args);

        if let Some(interval) = matches.value_of("interval") {
            self.interval = interval.parse().unwrap_or(0);
        }

        if matches.is_present("verbose") {
            self.verbose = true;
        }

        if let Some(warmup) = matches.value_of("warmup") {
            self.warmup = warmup.parse().unwrap_or(0);
        }

        if let Some(task_num) = matches.value_of("task_num") {
            self.task_num = task_num.parse().unwrap_or(1);
        }

        if let Some(test_type) = matches.value_of("test_type") {
            self.test_type = test_type.parse().unwrap_or(0);
        }

        if let Some(rideable_type) = matches.value_of("rideable_type") {
            self.rideable_type = rideable_type.parse().unwrap_or(0);
        }

        if let Some(affinity) = matches.value_of("affinity") {
            self.affinity = affinity.to_string();
        }

        if let Some(out_file) = matches.value_of("out_file") {
            self.out_file = Some(out_file.to_string());
        }
    }

    fn run_test(&mut self) {
        if self.warmup > 0 {
            warm_memory(self.warmup);
        }

        parallel_work(self);

        if let Some(out_file) = &self.out_file {
            if let Some(recorder) = &self.recorder {
                let mut file = File::create(out_file).expect("Failed to create output file");
                writeln!(file, "{}", recorder.get_csv()).expect("Failed to write output");
            }
        }
    }
}
