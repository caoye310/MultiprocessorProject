use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug)]
enum TrackerType {
    NIL,
    RCPU,
    Interval,
    Range,
    RangeNew,
    QSBR,
    Hazard,
    HazardDynamic,
    HE,
}


struct MemoryTracker<T> {
    tracker: Option<Box<dyn BaseTracker<T>>>,
    tracker_type: TrackerType,
    slot_renamers: Vec<Vec<usize>>,
}

impl<T> MemoryTracker<T> {
    fn new(task_num: usize, slot_num: usize, epoch_freq: usize, empty_freq: usize, collect: bool, tracker_type: String) -> Self {
        let mut tracker: Option<Box<dyn BaseTracker<T>>> = None;
        let tracker_type_enum = match tracker_type.as_str() {
            "NIL" => TrackerType::NIL,
            "RCU" => TrackerType::RCPU,
            "Interval" => TrackerType::Interval,
            "Range_new" => TrackerType::RangeNew,
            "Hazard" => TrackerType::Hazard,
            "HE" => TrackerType::HE,
            "QSBR" => TrackerType::QSBR,
            _ => panic!("Unknown tracker type"),
        };

        let slot_renamers = (0..task_num).map(|_| (0..slot_num).collect::<Vec<usize>>()).collect::<Vec<_>>();

        match tracker_type_enum {
            TrackerType::RCU => {
                // Instantiate the RCPU Tracker (similar to the C++ code).
                // tracker = Some(Box::new(RCPUTracker::new()));
            },
            TrackerType::Interval => {
                // Instantiate the Interval Tracker.
                // tracker = Some(Box::new(IntervalTracker::new()));
            },
            TrackerType::NIL => {
                // Instantiate NIL Tracker.
                // tracker = Some(Box::new(NilTracker::new()));
            },
            _ => {}
        }

        MemoryTracker {
            tracker,
            tracker_type: tracker_type_enum,
            slot_renamers,
        }
    }

    fn alloc(&self) -> *mut T {
        self.tracker.as_ref().expect("Tracker is not initialized").alloc()
    }

    fn alloc_tid(&self, tid: usize) -> *mut T {
        self.tracker.as_ref().expect("Tracker is not initialized").alloc_tid(tid)
    }

    fn reclaim(&self, obj: *mut T) {
        if let Some(tracker) = &self.tracker {
            tracker.reclaim(obj);
        }
    }

    fn reclaim_tid(&self, obj: *mut T, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.reclaim_tid(obj, tid);
        }
    }

    fn start_op(&self, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.start_op(tid);
        }
    }

    fn end_op(&self, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.end_op(tid);
        }
    }

    fn read(&self, obj: &std::sync::atomic::Atomic<T>, idx: usize, tid: usize) -> *mut T {
        self.tracker.as_ref().expect("Tracker is not initialized").read(obj, idx, tid)
    }

    fn transfer(&self, src_idx: usize, dst_idx: usize, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.transfer(src_idx, dst_idx, tid);
        }
    }

    fn release(&self, idx: usize, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.release(idx, tid);
        }
    }

    fn clear_all(&self, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.clear_all(tid);
        }
    }

    fn retire(&self, obj: *mut T, tid: usize) {
        if let Some(tracker) = &self.tracker {
            tracker.retire(obj, tid);
        }
    }

    fn get_retired_cnt(&self, tid: usize) -> u64 {
        if let Some(tracker) = &self.tracker {
            tracker.get_retired_cnt(tid)
        } else {
            0
        }
    }
}
