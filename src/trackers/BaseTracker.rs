use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::ptr;


trait BaseTracker<T> {
    fn new(task_num: usize) -> Self;
    fn get_retired_cnt(&self, tid: usize) -> u64;
    fn inc_retired(&self, tid: usize);
    fn dec_retired(&self, tid: usize);
    fn alloc(&self) -> *mut T;
    fn reclaim(&self, obj: *mut T);
    fn start_op(&self, tid: usize);
    fn end_op(&self, tid: usize);
    fn read(&self, obj: &AtomicU64, tid: usize) -> T;
    fn transfer(&self, src_idx: usize, dst_idx: usize, tid: usize);
    fn reserve(&self, obj: *mut T, idx: usize, tid: usize);
    fn release(&self, idx: usize, tid: usize);
    fn clear_all(&self, tid: usize);
    fn retire(&self, obj: *mut T, tid: usize);
}
