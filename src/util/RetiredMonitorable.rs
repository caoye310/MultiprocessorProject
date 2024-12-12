use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct GlobalTestConfig {
    task_num: usize,
}

struct RetiredMonitorable {
    retired_cnt: Vec<AtomicU64>, // A vector to hold AtomicU64 values for each thread
}

impl RetiredMonitorable {
    fn new(gtc: &GlobalTestConfig) -> Self {
        // Initialize the retired_cnt with AtomicU64 for each task_num
        let retired_cnt = (0..gtc.task_num)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<AtomicU64>>();

        RetiredMonitorable { retired_cnt }
    }

    // Collects the retired size for the given thread id
    fn collect_retired_size(&self, size: u64, tid: usize) {
        self.retired_cnt[tid].fetch_add(size, Ordering::SeqCst);
    }

    // Reports the retired size for the given thread id
    fn report_retired(&self, tid: usize) -> u64 {
        self.retired_cnt[tid].load(Ordering::SeqCst)
    }
}