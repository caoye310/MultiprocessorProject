use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::VecDeque;
use std::ptr;
mod util;
mod trackers;


struct IntervalTracker<T> {
    task_num: usize,
    freq: usize,
    epoch_freq: usize,
    collect: bool,
    retired: Vec<Padded<AtomicU64>>, // Replace list with atomic retired counters
    reservations: Vec<PaddedAtomic>,
    retire_counters: Vec<Padded<AtomicU64>>,
    alloc_counters: Vec<Padded<AtomicU64>>,
    epoch: AtomicU64,
}

impl<T> IntervalTracker<T> {
    fn new(task_num: usize, epoch_freq: usize, empty_freq: usize, collect: bool) -> Self {
        let retired = (0..task_num).map(|_| Padded { ui: AtomicU64::new(0) }).collect::<Vec<_>>();
        let reservations = (0..task_num).map(|_| PaddedAtomic { ui: AtomicU64::new(u64::MAX) }).collect::<Vec<_>>();
        let retire_counters = (0..task_num).map(|_| Padded { ui: AtomicU64::new(0) }).collect::<Vec<_>>();
        let alloc_counters = (0..task_num).map(|_| Padded { ui: AtomicU64::new(0) }).collect::<Vec<_>>();

        IntervalTracker {
            task_num,
            freq: empty_freq,
            epoch_freq,
            collect,
            retired,
            reservations,
            retire_counters,
            alloc_counters,
            epoch: AtomicU64::new(0),
        }
    }

    fn get_epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    fn alloc(&self, tid: usize) -> *mut T {
        self.alloc_counters[tid].ui.fetch_add(1, Ordering::SeqCst);
        if self.alloc_counters[tid].ui.load(Ordering::Acquire) % (self.epoch_freq as u64 * self.task_num as u64) == 0 {
            self.epoch.fetch_add(1, Ordering::AcqRel);
        }
        
        // Allocate memory for T and its associated birth epoch
        let block = Box::into_raw(Box::new(T::default()));
        let birth_epoch = unsafe { (block as *mut u8).offset(std::mem::size_of::<T>() as isize) as *mut u64 };
        unsafe { *birth_epoch = self.get_epoch() };

        block
    }

    fn read_birth(&self, obj: *mut T) -> u64 {
        unsafe {
            let birth_epoch = (obj as *mut u8).offset(std::mem::size_of::<T>() as isize) as *mut u64;
            *birth_epoch
        }
    }

    fn reclaim(&self, obj: *mut T) {
        if !obj.is_null() {
            unsafe { Box::from_raw(obj) }; // Automatically drops the object, reclaiming memory
        }
    }

    fn start_op(&self, tid: usize) {
        let e = self.get_epoch();
        self.reservations[tid].ui.store(e, Ordering::SeqCst);
    }

    fn end_op(&self, tid: usize) {
        self.reservations[tid].ui.store(u64::MAX, Ordering::SeqCst);
    }

    fn reserve(&self, tid: usize) {
        self.start_op(tid);
    }

    fn clear(&self, tid: usize) {
        self.end_op(tid);
    }

    fn validate(&self, tid: usize) -> bool {
        self.reservations[tid].ui.load(Ordering::Acquire) == self.get_epoch()
    }

    fn increment_epoch(&self) {
        self.epoch.fetch_add(1, Ordering::AcqRel);
    }

    fn retire(&self, obj: *mut T, birth_epoch: u64, tid: usize) {
        if obj.is_null() { return; }

        let retire_epoch = self.get_epoch();
        let info = IntervalInfo {
            obj,
            birth_epoch,
            retire_epoch,
        };

        let my_trash = &mut self.retired[tid].ui;
        my_trash.push_back(info);

        self.retire_counters[tid].ui.fetch_add(1, Ordering::SeqCst);

        if self.collect && self.retire_counters[tid].ui.load(Ordering::Acquire) % self.freq as u64 == 0 {
            self.empty(tid);
        }
    }

    fn retire_with_birth(&self, obj: *mut T, tid: usize) {
        self.retire(obj, self.read_birth(obj), tid);
    }

    fn conflict(&self, reserv_epoch: &[u64], birth_epoch: u64, retire_epoch: u64) -> bool {
        for &r_epoch in reserv_epoch {
            if r_epoch >= birth_epoch && r_epoch <= retire_epoch {
                return true;
            }
        }
        false
    }

    fn empty(&self, tid: usize) {
        let mut reserv_epoch = Vec::with_capacity(self.task_num);
        for i in 0..self.task_num {
            reserv_epoch.push(self.reservations[i].ui.load(Ordering::Acquire));
        }

        let my_trash = &mut self.retired[tid].ui;
        let mut i = 0;
        while i < my_trash.len() {
            let res = &my_trash[i];
            if !self.conflict(&reserv_epoch, res.birth_epoch, res.retire_epoch) {
                my_trash.remove(i);
                self.reclaim(res.obj);
                self.dec_retired(tid);
            } else {
                i += 1;
            }
        }
    }

    fn collecting(&self) -> bool {
        self.collect
    }
}

struct IntervalInfo<T> {
    obj: *mut T,
    birth_epoch: u64,
    retire_epoch: u64,
}

impl<T> IntervalInfo<T> {
    fn new(obj: *mut T, birth_epoch: u64, retire_epoch: u64) -> Self {
        IntervalInfo {
            obj,
            birth_epoch,
            retire_epoch,
        }
    }
}

impl Default for IntervalInfo<i32> {
    fn default() -> Self {
        IntervalInfo {
            obj: std::ptr::null_mut(),
            birth_epoch: 0,
            retire_epoch: 0,
        }
    }
}