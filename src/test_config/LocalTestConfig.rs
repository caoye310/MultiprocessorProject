pub struct LocalTestConfig {
    pub tid: i32,           // Equivalent to C++'s int
    pub seed: u32,          // Equivalent to C++'s unsigned int
    pub cpu: u32,           // Equivalent to C++'s unsigned cpu
    pub cpuset: Vec<u32>,   // Assuming cpuset is a collection of CPU IDs (bitmask or set)
}

impl LocalTestConfig {
    // Constructor for creating a new LocalTestConfig
    pub fn new(tid: i32, seed: u32, cpu: u32, cpuset: Vec<u32>) -> Self {
        LocalTestConfig {
            tid,
            seed,
            cpu,
            cpuset,
        }
    }
}