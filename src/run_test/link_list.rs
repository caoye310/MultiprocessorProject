use std::alloc::{alloc, dealloc, Layout};
use std::ptr::{null_mut};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;

// Node struct
struct Node<K, V> {
    key: K,
    value: V,
    next: AtomicPtr<Node<K, V>>,
}

// SortedUnorderedMap struct
pub(crate) struct SortedUnorderedMap<K, V> {
    buckets: Vec<AtomicPtr<Node<K, V>>>,
    bucket_count: usize,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V, next: *mut Node<K, V>) -> *mut Node<K, V> {
        unsafe {
            let layout = Layout::new::<Node<K, V>>();
            let ptr = alloc(layout) as *mut Node<K, V>;
            if ptr.is_null() {
                panic!("Failed to allocate memory for Node");
            }
            ptr.write(Node {
                key,
                value,
                next: AtomicPtr::new(next),
            });
            ptr
        }
    }

    unsafe fn dealloc(ptr: *mut Node<K, V>) {
        let layout = Layout::new::<Node<K, V>>();
        dealloc(ptr as *mut u8, layout);
    }
}

impl<K, V> SortedUnorderedMap<K, V>
where
    K: Ord + Hash + Clone + Debug,
    V: Clone + Debug,
{
    fn new(bucket_count: usize) -> Self {
        let mut buckets = Vec::with_capacity(bucket_count);
        for _ in 0..bucket_count {
            buckets.push(AtomicPtr::new(null_mut()));
        }
        SortedUnorderedMap { buckets, bucket_count }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.bucket_count
    }

    fn insert(&self, key: K, value: V) -> bool {
        let idx = self.hash(&key);
        let mut prev = &self.buckets[idx];
        let mut cur = prev.load(Ordering::SeqCst);

        loop {
            unsafe {
                if !cur.is_null() {
                    let cur_node = &*cur;
                    if cur_node.key >= key {
                        if cur_node.key == key {
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

        let new_node = Node::new(key, value, cur);

        // Try to insert using compare_exchange_strong for lock-free update
        if prev.compare_exchange(cur, new_node, Ordering::SeqCst, Ordering::Relaxed).is_err() {
            unsafe { Node::dealloc(new_node) }; // If the exchange fails, deallocate the node
            return false;
        }
        true
    }

    fn get(&self, key: &K) -> Option<V> {
        let idx = self.hash(key);
        let mut cur = self.buckets[idx].load(Ordering::SeqCst);

        while !cur.is_null() {
            unsafe {
                let cur_node = &*cur;
                if cur_node.key == *key {
                    return Some(cur_node.value.clone());
                } else if cur_node.key > *key {
                    break;
                }
                cur = cur_node.next.load(Ordering::SeqCst);
            }
        }
        None
    }

    fn remove(&self, key: &K) -> Option<V> {
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
                        Node::dealloc(cur);
                        return Some(value);
                    }
                } else if cur_node.key > *key {
                    break;
                }
                prev = &cur_node.next;
                cur = cur_node.next.load(Ordering::SeqCst);
            }
        }
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


fn testLinkList1Thread() {
    // 声明一个list
    let list = SortedUnorderedMap::new(1);

    list.insert(10, "Ten");
    list.insert(20, "Twenty");
    list.insert(15, "Fifteen");
    list.insert(5, "Five");
    list.insert(25, "Twenty-Five");

    println!("After insertion:");
    list.print();

    println!("Get key 15: {:?}", list.get(&15));
    println!("Get key 30: {:?}", list.get(&30));

    println!("Remove key 20: {:?}", list.remove(&20));

    println!("After removal:");
    list.print();
    let all_elements = list.load();
    println!("All elements in the map: {:?}", all_elements);
}