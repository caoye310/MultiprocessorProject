// A special Head tuple: HPtr: point to the head of the list. HRef: number of activate threads。
//  per-thread Handle variable: for each thread, store the snapshot of HPtr
// 每个节点都有两个字段：Next：指向列表中的下一个节点, NRef(可以访问这个节点的线程数)
// Figure 3 (b)为什么HRef
//
/*
use std::alloc::{Layout, alloc};
use std::sync::{Arc, Mutex};
use portable_atomic::AtomicI128;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[derive(Clone)]
struct Node {
    nref: AtomicU64,
    next: Option<Arc<Node>>,
}

// Head struct holds the retirement list's head pointer and the active thread counter (HRef)
struct Head {
    href: AtomicI128, // Atomic I128 for lock-free updates of href and hptr together
}

impl Head {
    fn new() -> Self {
        //let handle = Handle{ href_snapshot: 0, hptr_snapshot: None };
        Head {
            href: AtomicI128::new(0), // Initialize HRef and HPtr to 0
        }
    }
}

#[derive(Clone, Copy)]
struct Handle {
    href_snapshot: i64,
    hptr_snapshot: usize, // hptr serialized as usize
}

impl Handle {
    fn to_i128(self) -> i128 {
        ((self.href_snapshot as i128) << 64) | (self.hptr_snapshot as i128)
    }

    fn from_i128(value: i128) -> Self {
        let href = (value >> 64) as i64;
        let hptr = value as usize;
        Handle { href_snapshot: href, hptr_snapshot: hptr }
    }
}

pub(crate) struct MemoryTracker {
    head: Arc<Head>,
    num_threads: usize,
    retired: AtomicU64,
}

impl MemoryTracker {
    /// 创建一个新的链表
    pub(crate) fn new(num_threads:usize) -> Self {
        MemoryTracker {
            head: Arc::new(Head::new()),
            num_threads: num_threads,
            retired: AtomicU64::new(0),
        }
    }

    pub(crate) fn alloc<T>(&self, layout: Layout, pid:i32) -> *mut u8 {
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Memory allocation failed!");
            }
            ptr
        }
    }

    // 获取已退休的计数（每个任务的平均数）
    pub(crate) fn get_retired_cnt(&self, tid: i32) -> i64 {
        if self.num_threads > 0 {
            // 根据 task_num 计算平均每个任务的 retired 数
            self.retired.load(Ordering::Relaxed) as i64 / self.num_threads as i64
        } else {
            0
        }
    }

    /// 进入操作，增加引用计数并返回当前头指针的快照
    // Atomically increment HRef and return a snapshot of HPtr
    pub(crate) fn enter(&self) -> Handle {
        // Get the current value of HRef and HPtr atomically
        loop {
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // Create a snapshot handle
            let handle = Handle {
                href_snapshot: href,
                hptr_snapshot: hptr,
            };

            // Try to atomically increment HRef and update HPtr with the same value
            let new_href = href + 1;
            let new_value = ((new_href as i128) << 64) | (hptr as i128);

            // Perform a CAS operation to update the HRef and HPtr together using compare_exchange
            match self.head.href.compare_exchange(
                current,      // expected value
                new_value,    // new value
                Ordering::Release, // success ordering
                Ordering::Acquire, // failure ordering
            ) {
                Ok(_) => break handle, // Successfully updated, return the snapshot
                Err(_) => {} // CAS failed, retry
            }
        }
    }

    // Leave operation: decrement HRef and clean up any nodes if necessary
    fn leave(&self, handle: Option<Arc<Node>>) {
        loop {
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // Decrement HRef to indicate thread leaving
            let new_href = href - 1;

            // If no threads are left, set HPtr to None (or null equivalent)
            let new_hptr = if new_href == 0 { 0 } else { hptr };

            // Update HRef and HPtr atomically
            let new_value = ((new_href as i128) << 64) | (new_hptr as i128);

            // Perform a CAS operation to update HRef and HPtr together using compare_exchange
            match self.head.href.compare_exchange(
                current,      // expected value
                new_value,    // new value
                Ordering::Release, // success ordering
                Ordering::Acquire, // failure ordering
            ) {
                Ok(_) => break, // Successfully updated, exit the loop
                Err(_) => {} // CAS failed, retry
            }
        }

        // Clean up nodes after the thread leaves
        if let Some(node) = handle {
            let mut current = node;
            while let Some(n) = current {
                let mut n_lock = n.clone();  // Clone the node for lock-free operation
                n_lock.nref.fetch_sub(1, Ordering::Release);
                if n_lock.nref.load(Ordering::Acquire) == 0 {
                    // If the node's reference count reaches 0, it can be retired
                    break;
                }
                current = n_lock.next.clone();
            }
        }
    }

    /// 插入新节点并更新链表
    fn retire(&self, new_node: Arc<Mutex<Node>>) {
        loop {
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // Insert a new node into the linked list
            let mut node_lock = new_node.clone();
            node_lock.nref.store(0, Ordering::Release); // No references to the new node yet
            node_lock.next = None;

            // Update the linked list
            let new_value = ((href as i128) << 64) | (hptr as i128);

            match self.head.href.compare_exchange(
                current,      // expected value
                new_value,    // new value
                Ordering::Release, // success ordering
                Ordering::Acquire, // failure ordering
            ) {
                Ok(_) => break, // Successfully retired, exit the loop
                Err(_) => {} // CAS failed, retry
            }
        }
    }

    /// 遍历链表，应用用户提供的操作
    fn traverse<F>(&self, mut func: F)
    where
        F: FnMut(&Node),
    {
        let head = self.head.lock().unwrap();
        let mut current = head.hptr.clone();
        drop(head); // 解锁头部，允许其他线程操作

        while let Some(node) = current {
            let node_lock = node.lock().unwrap();
            func(&*node_lock);
            current = node_lock.next.clone();
        }
    }
}*/
use std::sync::atomic::{AtomicU64, Ordering};
use portable_atomic::AtomicI128;
use std::sync::Arc;
use std::alloc::{Layout, alloc, dealloc};

// Node struct representing an individual element in the linked list
struct Node {
    nref: AtomicU64,    // Atomic reference count (for lock-free decrement)
    next: Option<Arc<Node>>, // Next node in the linked list
}

// Head struct holds the retirement list's head pointer and the active thread counter (HRef)
struct Head {
    href: AtomicI128, // Atomic I128 for lock-free updates of href and hptr together
}

impl Head {
    fn new() -> Self {
        Head {
            href: AtomicI128::new(0), // Initialize HRef and HPtr to 0
        }
    }
}

// Handle struct to store a snapshot of HPtr for each thread
#[derive(Clone, Copy)]
pub(crate) struct Handle {
    href_snapshot: i64,
    hptr_snapshot: usize, // hptr serialized as usize
}

impl Handle {
    fn to_i128(self) -> i128 {
        ((self.href_snapshot as i128) << 64) | (self.hptr_snapshot as i128)
    }

    fn from_i128(value: i128) -> Self {
        let href = (value >> 64) as i64;
        let hptr = value as usize;
        Handle { href_snapshot: href, hptr_snapshot: hptr }
    }
}

// MemoryTracker holds the global retirement list and provides methods for interacting with it
pub(crate) struct MemoryTracker {
    head: Arc<Head>, // Arc to share the Head structure across threads
    num_threads: usize,
    retired: AtomicU64,
}

impl MemoryTracker {
    // Create a new MemoryTracker instance
    pub(crate) fn new(num_threads: usize) -> Self {
        MemoryTracker {
            head: Arc::new(Head::new()),
            num_threads,
            retired: AtomicU64::new(0),
        }
    }

    // Memory allocation (for illustration, doesn't use in `enter` function)
    pub(crate) fn alloc<T>(&self, layout: Layout, pid: i32) -> *mut u8 {
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Memory allocation failed!");
            }
            ptr
        }
    }

    pub(crate) fn dealloc<T>(&self, ptr: *mut u8, layout: Layout) {
        unsafe { dealloc(ptr, layout) };
    }

    // Atomically increment HRef and return a snapshot of HPtr
    pub(crate) fn enter(&self) -> Handle {
        loop {
            // Get the current value of HRef and HPtr atomically
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // Create a snapshot handle
            let handle = Handle {
                href_snapshot: href,
                hptr_snapshot: hptr,
            };

            // Try to atomically increment HRef and update HPtr with the same value
            let new_href = href + 1;
            let new_value = ((new_href as i128) << 64) | (hptr as i128);

            // Perform a CAS operation to update the HRef and HPtr together using compare_exchange
            match self.head.href.compare_exchange(
                current,      // expected value
                new_value,    // new value
                Ordering::Release, // success ordering
                Ordering::Acquire, // failure ordering
            ) {
                Ok(_) => break handle, // Successfully updated, return the snapshot
                Err(_) => {} // CAS failed, retry
            }
        }
    }

    // Get the retired count (average number of retired nodes per thread)
    pub(crate) fn get_retired_cnt(&self, tid: i32) -> i64 {
        if self.num_threads > 0 {
            self.retired.load(Ordering::Relaxed) as i64 / self.num_threads as i64
        } else {
            0
        }
    }

    // Leave operation: decrement HRef and clean up any nodes if necessary
    pub(crate) fn leave(&self, handle: Option<Arc<Node>>) {
        loop {
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // Decrement HRef to indicate thread leaving
            let new_href = href - 1;

            // If no threads are left, set HPtr to None (or null equivalent)
            let new_hptr = if new_href == 0 { 0 } else { hptr };

            // Update HRef and HPtr atomically
            let new_value = ((new_href as i128) << 64) | (new_hptr as i128);

            // Perform a CAS operation to update HRef and HPtr together using compare_exchange
            match self.head.href.compare_exchange(
                current,      // expected value
                new_value,    // new value
                Ordering::Release, // success ordering
                Ordering::Acquire, // failure ordering
            ) {
                Ok(_) => break, // Successfully updated, exit the loop
                Err(_) => {} // CAS failed, retry
            }
        }

        // Clean up nodes after the thread leaves
        if let Some(node) = handle {
            let mut current = node;
            while let Some(n) = current {
                let mut n_lock = n.clone();  // Clone the node for lock-free operation
                n_lock.nref.fetch_sub(1, Ordering::Release);
                if n_lock.nref.load(Ordering::Acquire) == 0 {
                    // If the node's reference count reaches 0, it can be retired
                    break;
                }
                current = n_lock.next.clone();
            }
        }
    }

    // Retire operation: Insert new nodes into the linked list
    fn retire(&self, new_node: Arc<Node>) {
        loop {
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // Insert a new node into the linked list

            let mut node_lock = new_node.clone();
            node_lock.nref.store(0, Ordering::Release); // No references to the new node yet
            node_lock.next = None;


            // Update the linked list
            let new_value = ((href as i128) << 64) | (hptr as i128);

            match self.head.href.compare_exchange(
                current,      // expected value
                new_value,    // new value
                Ordering::Release, // success ordering
                Ordering::Acquire, // failure ordering
            ) {
                Ok(_) => break, // Successfully retired, exit the loop
                Err(_) => {} // CAS failed, retry
            }
        }
    }

    // Traverse the linked list and apply a function to each node
    fn traverse<F>(&self, mut func: F)
    where
        F: FnMut(&Node),
    {
        // Atomically load the head pointer (HPtr) from the global Head structure
        let head_value = self.head.href.load(Ordering::Acquire);
        let hptr = head_value as usize;

        // If the head pointer is null, the list is empty
        if hptr == 0 {
            return;
        }

        // Start traversal from the first node
        let mut current = unsafe { Arc::from_raw(hptr as *const Node) };

        while let Some(node) = Arc::get_mut(&mut current) {
            // Apply the user-provided function to the current node
            func(node);

            // Move to the next node in the list
            if let Some(next) = &node.next {
                current = next.clone(); // Move to the next node
            } else {
                break; // End of the list
            }
        }
    }
}

