use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::time::SystemTime;
use std::time::Duration;
use libc::{sysconf, _SC_PAGESIZE};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::File;
use std::sync::{Arc, Mutex};
use sysinfo::{System, SystemExt};
use std::io::Write;
use std::thread;
use std::time::{Instant};
use trackers::IntervalTrackers;
use util::ConcurrentPrimitives;
use util::HarnessUtils;
use util::RAllocator;
use util::RetiredMonitorable;
use util::Rideable;

pub struct GlobalTestConfig {
    task_num: i32, // number of threads
    start: SystemTime, // timing structures
    finish: SystemTime, // timing structures
    interval: u64, // number of seconds to run test
    affinities: Vec<hwloc_obj_t>, // map from tid to CPU id
    topology: hwloc_topology_t,
    num_procs: i32,
    test: Option<Test>, // Option to handle nullability
    test_type: i32,
    rideable_type: i32,
    verbose: i32,
    warmup: i32, // MBs of data to warm
    timeout: bool, // whether to abort on infinite loop
    affinity: String,
    recorder: Option<Recorder>,
    rideable_factories: Vec<RideableFactory>,
    rideable_names: Vec<String>,
    tests: Vec<Test>,
    test_names: Vec<String>,
    out_file: String,
    allocated_rideables: Vec<Rideable>,
    total_operations: i64,
    environment: HashMap<String, String>,
    arguments: HashMap<String, *mut std::ffi::c_void>,
    argv0: String,
}

impl GlobalTestConfig {
    // Constructor
    pub fn new() -> Self {
        GlobalTestConfig {
            test_names: Vec::new(),
            out_file: String::new(),
            task_num: i32,
        }
    }

    // For accessing rideable objects
    // Allocates a Rideable using the selected RideableFactory
    pub fn alloc_rideable(&mut self) -> Option<Rideable> {
        if let Some(factory) = self.rideable_factories.get(self.rideable_type as usize) {
            let r = factory.build(self); // Assume the `build` method takes `self` as an argument
            self.allocated_rideables.push(r);
            Some(r)
        } else {
            None // Return None if the rideable factory index is out of bounds
        }
    }

    // pub fn parse_command_line(&mut self, args: Vec<String>) {
    //     let mut arg_iter = args.iter().skip(1); // Skip the program name
    //     self.argv0 = args[0].clone();

    //     // If no args, print help
    //     if args.len() == 1 {
    //         self.print_arg_def();
    //         self.errexit("");
    //     }

    //     // If no test options, error out
    //     if self.tests.is_empty() {
    //         self.errexit("No test options provided. Use GlobalTestConfig::addTestOption() to add.");
    //     }

    //     // Read command line
    //     while let Some(arg) = arg_iter.next() {
    //         match arg.as_str() {
    //             "-i" => {
    //                 self.interval = arg_iter.next().unwrap().parse().unwrap();
    //             }
    //             "-v" => {
    //                 self.verbose = true;
    //             }
    //             "-w" => {
    //                 self.warmup = arg_iter.next().unwrap().parse().unwrap();
    //             }
    //             "-t" => {
    //                 self.task_num = arg_iter.next().unwrap().parse().unwrap();
    //             }
    //             "-m" => {
    //                 self.test_type = arg_iter.next().unwrap().parse().unwrap();
    //                 if self.test_type >= self.tests.len() {
    //                     eprintln!("Invalid test mode (-m) option.");
    //                     self.print_arg_def();
    //                 }
    //             }
    //             "-r" => {
    //                 self.rideable_type = arg_iter.next().unwrap().parse().unwrap();
    //                 if self.rideable_type >= self.rideable_factories.len() {
    //                     eprintln!("Invalid rideable (-r) option.");
    //                     self.print_arg_def();
    //                 }
    //             }
    //             "-a" => {
    //                 self.affinity = arg_iter.next().unwrap().to_string();
    //             }
    //             "-h" => {
    //                 self.print_arg_def();
    //             }
    //             "-o" => {
    //                 self.out_file = arg_iter.next().unwrap().to_string();
    //             }
    //             "-z" => {
    //                 self.time_out = false;
    //             }
    //             "-d" => {
    //                 let env_var = arg_iter.next().unwrap();
    //                 let parts: Vec<&str> = env_var.splitn(2, '=').collect();
    //                 let (key, value) = if parts.len() == 2 {
    //                     (parts[0].to_string(), parts[1].to_string())
    //                 } else {
    //                     (parts[0].to_string(), "1".to_string())
    //                 };

    //                 let value = match value.as_str() {
    //                     "true" => "1".to_string(),
    //                     "false" => "0".to_string(),
    //                     _ => value,
    //                 };

    //                 self.environment.insert(key, value);
    //             }
    //             _ => {}
    //         }
    //     }

    //     self.test = Some(self.tests[self.test_type].clone());

    //     // Initialize hwloc topology
    //     self.hwloc_topology_init();
    //     self.hwloc_topology_load();
    //     self.num_procs = self.hwloc_get_nbobjs_by_depth();

    //     self.build_affinity();

    //     // Setup recorder
    //     self.recorder = Some(Box::new(Recorder::new(self.task_num)));
    //     self.recorder.as_mut().unwrap().report_global_info("datetime", Recorder::date_time_string());
    //     self.recorder.as_mut().unwrap().report_global_info("threads", self.task_num.to_string());
    //     self.recorder.as_mut().unwrap().report_global_info("cores", self.num_procs.to_string());
    //     self.recorder.as_mut().unwrap().report_global_info("rideable", self.get_rideable_name());
    //     self.recorder.as_mut().unwrap().report_global_info("affinity", self.affinity.clone());
    //     self.recorder.as_mut().unwrap().report_global_info("test", self.get_test_name());
    //     self.recorder.as_mut().unwrap().report_global_info("interval", self.interval.to_string());
    //     self.recorder.as_mut().unwrap().report_global_info("language", "Rust".to_string());
    //     self.recorder.as_mut().unwrap().report_global_info("machine", self.machine_name());
    //     self.recorder.as_mut().unwrap().report_global_info("archbits", self.arch_bits());
    //     self.recorder.as_mut().unwrap().report_global_info("preheated(MBs)", self.warmup.to_string());
    //     self.recorder.as_mut().unwrap().report_global_info("notes", "".to_string());
    //     self.recorder.as_mut().unwrap().add_thread_field("ops", &Recorder::sum_ints);
    //     self.recorder.as_mut().unwrap().add_thread_field("ops_stddev", &Recorder::std_dev_ints);
    //     self.recorder.as_mut().unwrap().add_thread_field("ops_each", &Recorder::concat);

    //     // Report environment variables
    //     let env = self.environment.iter()
    //         .map(|(k, v)| format!("{}={}", k, v))
    //         .collect::<Vec<String>>()
    //         .join(":");

    //     if !env.is_empty() {
    //         self.recorder.as_mut().unwrap().report_global_info("environment", env);
    //     }

    //     // Verbose environment output
    //     if self.verbose && !self.environment.is_empty() {
    //         println!("Using flags:");
    //         for (key, value) in &self.environment {
    //             println!("{} = \"{}\"", key, value);
    //         }
    //     }
    // }

    // Get the rideable name at the specified index
    pub fn get_rideable_name(&self) -> String {
        self.rideable_names[self.rideable_type].clone()
    }

    // Add a rideable option
    pub fn add_rideable_option(&mut self, h: Box<dyn RideableFactory>, name: &str) {
        self.rideable_factories.push(h);
        self.rideable_names.push(name.to_string());
    }

    // Get the test name at the specified index
    pub fn get_test_name(&self) -> String {
        self.test_names[self.test_type].clone()
    }

    // Add a test option
    pub fn add_test_option(&mut self, t: Box<dyn Test>, name: &str) {
        self.tests.push(t);
        self.test_names.push(name.to_string());
    }

    // Sets an environment variable with a key-value pair
    pub fn set_env(&mut self, key: String, value: String) {
        if self.verbose {
            println!("setEnv: {} = \"{}\"", key, value);
        }
        self.environment.insert(key, value);
    }

    // Checks if the environment variable exists and its value is non-empty and not "0"
    pub fn check_env(&self, key: &str) -> bool {
        if self.verbose {
            println!("checkEnv: {}", key);
        }
        match self.environment.get(key) {
            Some(value) => !value.is_empty() && value != "0",
            None => false,
        }
    }

    // Retrieves the environment variable value
    pub fn get_env(&self, key: &str) -> String {
        if self.verbose {
            println!("getEnv: {}", key);
        }
        self.environment.get(key).cloned().unwrap_or_default()
    }

    // Sets an argument with a key-value pair, where value is a boxed Any type
    pub fn set_arg(&mut self, key: String, value: Box<dyn Any>) {
        if self.verbose {
            println!("setArg: {} = \"{:?}\"", key, value);
        }
        self.arguments.insert(key, value);
    }

    // Checks if the argument exists and is non-null
    pub fn check_arg(&self, key: &str) -> bool {
        if self.verbose {
            println!("checkArg: {}", key);
        }
        self.arguments.contains_key(key)
    }

    // Retrieves the argument value
    pub fn get_arg<T: 'static>(&self, key: &str) -> Option<&T> {
        if self.verbose {
            println!("getArg: {}", key);
        }
        self.arguments.get(key).and_then(|value| value.downcast_ref::<T>())
    }

    // Run the test
    pub fn run_test(&mut self) {
        if self.warmup != 0 {
            self.warm_memory(self.warmup);
        }

        self.parallel_work();

        // if !self.out_file.is_empty() {
        //     if let Some(ref mut recorder) = self.recorder {
        //         recorder.output_to_file(&self.out_file);
        //         if self.verbose {
        //             println!("Stored test results in: {}", self.out_file);
        //         }
        //     }
        // }

        // if let Some(ref recorder) = self.recorder {
        //     if self.verbose {
        //         println!("{}", recorder.get_csv());
        //     }
        // }

        let mut handles = vec![];
        let start = Instant::now();
        let number_of_threads = self.num_threads;
        let contain_percent = self.contain_percent;
        // Wrap self in an Arc and Mutex for safe shared ownership
        let self_arc = Arc::new(Mutex::new(self));  // Wrap `self` in an Arc<Mutex<YourStruct>>

        let memory_data = Arc::new(Mutex::new(Vec::new()));
        let system_arc = Arc::new(Mutex::new(System::new_all()));

        let memory_data_clone = Arc::clone(&memory_data);
        let system_clone = Arc::clone(&system_arc);
        let monitor_handle = thread::spawn(move || {
            for _ in 0..(5.0/0.02) as usize { // 1秒钟的采样次数，(1.0 / 0.001) 代表每秒 1000 次
                {
                    let mut system = system_clone.lock().unwrap();

                    // Get system's available memory
                    match sys_info::mem_info() {
                        Ok(info) => {
                            let mut data = memory_data_clone.lock().unwrap();
                            data.push(info.avail);
                            //println!("Available memory: {} bytes", info.avail);
                        },
                        Err(e) => eprintln!("Error fetching memory info: {}", e)
                    }

                }
                thread::sleep(Duration::from_millis(20));
            }
        });

        for i in 0..number_of_threads {
            let pid = i;  // thread id
            //let seed: u64 = rand::thread_rng().random();

            let thread_info = ThreadInfo { thread_id: pid};
            let self_clone = Arc::clone(&self_arc); // Clone the Arc to share ownership across threads
            // Create a thread and give it thread_info
            let handle = thread::spawn(move || {
                let self_locked = self_clone.lock().unwrap();
                self_locked.thread_main(thread_info, contain_percent);  // Each thread run `thread_main` independently
            });

            handles.push(handle);
        }

        // Waiting for all the threads finish
        for handle in handles {
            handle.join().unwrap();
        }
        monitor_handle.join().unwrap();

        let data_file = "data.csv";
        let mut file = File::create(data_file).expect("file creation error");
        let memory_data = memory_data.lock().unwrap();
        for (i, &value) in memory_data.iter().enumerate() {
            writeln!(file, "{},{:.2}", i as f64 * 0.2, value as i64).expect("error writing memory data");
        }
    }

    // Builds the affinity based on the chosen method (dfs, single, or default)
    pub fn build_affinity(&mut self) {
        match self.affinity.as_str() {
            "dfs" => self.build_dfs_affinity(),
            "single" => self.build_single_affinity(),
            _ => self.build_default_affinity(),
        }

        // Ensure affinities vector has at least task_num elements
        if self.affinities.len() < self.task_num {
            self.affinities.resize(self.task_num, self.affinities[0].clone());
        }

        // Reuse values cyclically for remaining elements in affinities
        for i in self.num_procs..self.task_num {
            self.affinities[i] = self.affinities[i % self.num_procs].clone();
        }
    }


    // Helper function to traverse the affinity tree recursively
    pub fn build_dfs_affinity_helper(&mut self, obj: &hwloc_obj_t) {
        if obj.obj_type == HWLOC_OBJ_PU {
            self.affinities.push(obj.clone()); // Add the object to affinities
            return;
        }
        if self.affinities.len() >= self.task_num as usize {
            return; // Stop if the number of affinities reaches the task number
        }
        for child in &obj.children {
            self.build_dfs_affinity_helper(child); // Recursively visit children
        }
    }

    // Top-level function to start the DFS traversal from the root
    pub fn build_dfs_affinity(&mut self) {
        let root_obj = self.get_root_obj(); // Assuming this is implemented elsewhere to get the root object
        self.build_dfs_affinity_helper(&root_obj);
    }

    // Recursively finds cores in a socket and adds them to the `cores` vector
    pub fn build_default_affinity_find_cores_in_socket(&mut self, obj: &hwloc_obj_t, cores: &mut Vec<hwloc_obj_t>) -> i32 {
        if obj.obj_type == HWLOC_OBJ_CORE {
            cores.push(obj.clone()); // Add the core to the cores vector
            return 1;
        }
        if obj.obj_type == HWLOC_OBJ_PU {
            return 0; // Error: should not reach PUs before cores
        }
        let mut ret = 1;
        for child in &obj.children {
            ret &= self.build_default_affinity_find_cores_in_socket(child, cores); // Recursive call to child nodes
        }
        ret
    }

    // Builds PUs (Processing Units) within the cores, filling the `affinities` vector
    pub fn build_default_affinity_build_pus_in_cores(&mut self, cores: &mut Vec<hwloc_obj_t>) -> i32 {
        let mut core_index = 0;
        let mut cores_filled = 0;
        while cores_filled < cores.len() {
            for i in 0..cores.len() {
                if core_index == cores[i].arity {
                    cores_filled += 1;
                    continue;
                }
                let obj = &cores[i].children[core_index];
                if obj.obj_type != HWLOC_OBJ_PU {
                    return 0; // Error: expected a PU
                }
                self.affinities.push(obj.clone()); // Add the PU to the affinities vector
            }
            core_index += 1;
        }
        1
    }

    // Recursively finds and builds sockets, cores, and PUs
    pub fn build_default_affinity_find_and_build_sockets(&mut self, obj: &hwloc_obj_t) -> i32 {
        // Recursion terminates at sockets
        if obj.obj_type == HWLOC_OBJ_SOCKET {
            let mut cores = Vec::new();
            if self.build_default_affinity_find_cores_in_socket(obj, &mut cores) == 0 {
                return 0; // Couldn't find cores in this socket, flag error
            }
            // Now "cores" is filled with all cores below the socket,
            // so assign threads to this core
            return self.build_default_affinity_build_pus_in_cores(&mut cores);
        }
        // Recursion down by DFS for children
        let mut ret = 1;
        for child in &obj.children {
            ret &= self.build_default_affinity_find_and_build_sockets(child); // Recursive call
        }
        ret
    }

    // Builds the default affinity (sockets, cores, PUs) or falls back to DFS
    pub fn build_default_affinity(&mut self) {
        if self.build_default_affinity_find_and_build_sockets(&self.hwloc_root_obj) == 0 {
            eprintln!("Unsupported topology for default thread pinning (unable to detect sockets and cores).");
            eprintln!("Switching to depth first search affinity.");
            self.affinities.clear(); // Clear affinities
            self.build_dfs_affinity(); // Fall back to DFS affinity
        }
    }

    // Recursively builds single affinity by traversing through the topology
    pub fn build_single_affinity_helper(&mut self, obj: &hwloc_obj_t) {
        if obj.obj_type == HWLOC_OBJ_PU {
            // Add the PU to the affinities vector task_num times
            for _ in 0..self.task_num {
                self.affinities.push(obj.clone());
            }
            return;
        }
        // Recursively call the first child (the main child) of the object
        if let Some(first_child) = obj.children.first() {
            self.build_single_affinity_helper(first_child);
        }
    }

    // Builds the single affinity by starting from the root of the topology
    pub fn build_single_affinity(&mut self) {
        self.build_single_affinity_helper(&self.hwloc_root_obj);
    }

    pub fn create_test(&mut self) {
        // Placeholder: Implementation will be provided when methods are given
    }

    pub fn print_arg_def(&self) {
        // Print usage information
        eprintln!("usage: {} [-m <test_mode>] [-r <rideable_test_object>] [-a single|dfs|default] [-i <interval>] [-t <num_threads>] [-o <output_csv_file>] [-w <warm_up_MBs>] [-d <env_variable>=<value>] [-z] [-v] [-h]", self.argv0);

        // Print the rideable names
        for (i, rideable_name) in self.rideable_names.iter().enumerate() {
            eprintln!("Rideable {} : {}", i, rideable_name);
        }

        // Print the test names
        for (i, test_name) in self.test_names.iter().enumerate() {
            eprintln!("Test Mode {} : {}", i, test_name);
        }

        // Exit the program after printing the usage
        std::process::exit(0);
    }
}