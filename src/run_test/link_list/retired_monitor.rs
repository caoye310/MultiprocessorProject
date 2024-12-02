use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
pub(crate) mod Hyaline;

use Hyaline::MemoryTracker;

pub(crate) struct RetiredMonitorable {
    retired_cnt: Vec<AtomicI64>,
    mem_tracker: Option<Arc<MemoryTracker>>, // 使用 Option 来表示可能为空的引用
}

impl RetiredMonitorable {
    pub(crate) fn new(num_threads:usize) -> Self {
        let retired_cnt = (0..num_threads)
            .map(|_| AtomicI64::new(0))
            .collect();

        RetiredMonitorable {
            retired_cnt,
            mem_tracker: None,
        }
    }

    fn set_base_mt(&mut self, base: Arc<MemoryTracker>) {
        self.mem_tracker = Some(base); // some 避免对null的判断，some函数内的数必须有实际的值
    }

    fn collect_retired_size(&self, size: i64, tid: usize) {
        // fetch_add对变量进行原子加法，Ordering::SeqCst：内存序列化模式，表示此操作的内存屏障规则。
        // SeqCst（顺序一致性）是最强的内存序列化模式，意味着该操作在所有线程中按顺序发生，并且之前的所有操作都被"同步"。
        // 这确保了所有线程看到的更新顺序一致。
        self.retired_cnt[tid].fetch_add(size, Ordering::SeqCst);
    }

    fn report_retired(&self, tid: usize) -> i64 {
        // 调用此函数时报告退役的数量
        if let Some(mem_tracker) = &self.mem_tracker {
            mem_tracker.last_exit(tid);
        }
        self.retired_cnt[tid].load(Ordering::SeqCst)
    }
}