use std::alloc::Layout;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
mod Hyaline;

use Hyaline::{MemoryTracker, Node, Handle};

// Node struct

// SortedUnorderedMap struct
pub(crate) struct SortedUnorderedMap<K, V> {
    tracker: MemoryTracker<K,V>,
    buckets: Vec<AtomicPtr<Node<K, V>>>,
    //monitor: RetiredMonitorable,
    handles: Vec<AtomicPtr<Handle<K,V>>>,
    layout: Layout,
    bucket_count: usize,
}

impl<K, V> SortedUnorderedMap<K, V>
where
    K: Ord + Hash + Clone + Debug,
    V: Clone + Debug,
{
    pub(crate) fn new(bucket_count: usize, num_threads:i32) -> Self {
        let mut buckets = Vec::with_capacity(bucket_count);
        let mut tracker = MemoryTracker::new(num_threads);
        //let mut monitor = RetiredMonitorable::new(num_threads);
        let mut handles = Vec::with_capacity(num_threads as usize);
        for _ in 0..bucket_count {
            buckets.push(AtomicPtr::new(null_mut()));
        }
        let layout = Layout::new::<Node<K, V>>();
        SortedUnorderedMap {tracker, buckets, handles, layout, bucket_count}
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.bucket_count
    }

    pub(crate) fn insert(&self, key: K, value: V, tid:i32) -> bool {
        self.handles[tid] = self.tracker.enter();
        let idx = self.hash(&key);
        let mut prev = &self.buckets[idx];
        let mut cur = prev.load(Ordering::SeqCst);
        let new_node = Node::new(key, value, cur);

        loop {
            unsafe {
                if !cur.is_null() {
                    let cur_node = &*cur;
                    if cur_node.key >= key {
                        if cur_node.key == key {
                            self.tracker.leave(self.handles[tid].hptr_snapshot);
                            return false; // Duplicate key found
                        }
                        break; // Found the insertion point
                    }
                    prev = &cur_node.next;
                    cur = cur_node.next.load(Ordering::SeqCst);
                } else {
                    break; // Insert at the end of the list
                }
            }
        }

        if prev.compare_exchange(cur, new_node, Ordering::SeqCst, Ordering::Relaxed).is_err() {
            let ptr = new_node as *mut u8;
            self.tracker.dealloc(ptr, self.layout);// If the exchange fails, deallocate the node
            self.tracker.leave(self.handles[tid].hptr_snapshot);
            return false;
        }
        self.tracker.leave(self.handles[tid].hptr_snapshot);
        true
    }

    fn get(&self, key: &K, tid:i32) -> Option<V> {
        self.handles[tid] = self.tracker.enter();
        let idx = self.hash(key);
        let mut cur = self.buckets[idx].load(Ordering::SeqCst);

        while !cur.is_null() {
            unsafe {
                let cur_node = &*cur;
                if cur_node.key == *key {
                    self.tracker.leave(self.handles[tid].hptr_snapshot);
                    return Some(cur_node.value.clone());
                } else if cur_node.key > *key {
                    break;
                }
                cur = cur_node.next.load(Ordering::SeqCst);
            }
        }
        self.tracker.leave(self.handles[tid].hptr_snapshot);
        None
    }

    pub(crate) fn remove(&self, key: &K, tid:i32) -> Option<V> {
        self.handles[tid] = self.tracker.enter();
        let idx = self.hash(key);
        let mut prev = &self.buckets[idx];
        let mut cur = prev.load(Ordering::SeqCst);

        while !cur.is_null() {
            unsafe {
                let cur_node = &*cur;
                if cur_node.key == *key {
                    let next = cur_node.next.load(Ordering::SeqCst);
                    if prev.compare_exchange(cur, next, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                        let value = cur_node.value.clone();
                        //Node::dealloc(cur);
                        self.tracker.dealloc(cur_node as *mut u8, self.layout);// If the exchange fails, deallocate the node
                        self.tracker.leave(self.handles[tid].hptr_snapshot);
                        return Some(value);
                    }
                } else if cur_node.key > *key {
                    break;
                }
                prev = &cur_node.next;
                cur = cur_node.next.load(Ordering::SeqCst);
            }
        }
        self.tracker.leave(self.handles[tid].hptr_snapshot);
        None
    }

    fn load(&self) -> Vec<(K, V)> {
        let mut result = Vec::new();
        for bucket in &self.buckets {
            let mut cur = bucket.load(Ordering::SeqCst);
            while !cur.is_null() {
                unsafe {
                    let cur_node = &*cur;
                    result.push((cur_node.key.clone(), cur_node.value.clone()));
                    cur = cur_node.next.load(Ordering::SeqCst);
                }
            }
        }
        result
    }

    fn print(&self) {
        for (i, bucket) in self.buckets.iter().enumerate() {
            print!("Bucket {}: ", i);
            let mut cur = bucket.load(Ordering::SeqCst);
            while !cur.is_null() {
                unsafe {
                    let cur_node = &*cur;
                    print!("({:?}, {:?}) -> ", cur_node.key, cur_node.value);
                    cur = cur_node.next.load(Ordering::SeqCst);
                }
            }
            println!("null");
        }
    }
}


// fn testLinkList1Thread() {
//     // 声明一个list
//     let list = SortedUnorderedMap::new(1);
//
//     list.insert(10, "Ten");
//     list.insert(20, "Twenty");
//     list.insert(15, "Fifteen");
//     list.insert(5, "Five");
//     list.insert(25, "Twenty-Five");
//
//     println!("After insertion:");
//     list.print();
//
//     println!("Get key 15: {:?}", list.get(&15));
//     println!("Get key 30: {:?}", list.get(&30));
//
//     println!("Remove key 20: {:?}", list.remove(&20));
//
//     println!("After removal:");
//     list.print();
//     let all_elements = list.load();
//     println!("All elements in the map: {:?}", all_elements);
// }