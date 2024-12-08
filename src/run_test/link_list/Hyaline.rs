// A special Head tuple: HPtr: point to the head of the list. HRef: number of activate threads。
//  per-thread Handle variable: for each thread, store the snapshot of HPtr
// 每个节点都有两个字段：Next：指向列表中的下一个节点, NRef(可以访问这个节点的线程数)
// Figure 3 (b)为什么HRef
//
use std::sync::atomic::{AtomicI64, AtomicPtr, AtomicU64, Ordering};
use portable_atomic::AtomicI128;
use std::sync::Arc;
use std::alloc::{Layout, alloc, dealloc};
use std::ptr;

use std::ptr::null_mut;
use std::thread::current;
struct MyAlloc {}
impl MyAlloc {
    fn new() -> MyAlloc {
        MyAlloc {}
    }

    pub(crate) fn alloc<T>(&self, layout: Layout) -> *mut u8 {
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

}


pub(crate) struct Node<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
    nref: AtomicI64,    // Atomic reference count (for lock-free decrement)
    pub(crate) next: AtomicPtr<Node<K, V>>,
    // mem_manage: MyAlloc,
}

impl<K, V> Node<K, V> {
    pub(crate) fn new(key: K, value: V, next: *mut Node<K, V>) -> *mut Node<K, V> {
        unsafe {
            let layout = Layout::new::<Node<K, V>>();
            let mem_manage= MyAlloc::new();
            let ptr = mem_manage.alloc(layout) as *mut Node<K, V>; // 全局变量？
            if ptr.is_null() {
                panic!("Failed to allocate memory for Node");
            }
            ptr.write(Node {
                key,
                value,
                nref: AtomicI64::new(0),
                next: AtomicPtr::new(next),
            });
            ptr
        }
    }

    unsafe fn dealloc(ptr: *mut Node<K, V>) {
        let layout = Layout::new::<Node<K, V>>();
        dealloc(ptr as *mut u8, layout);
    }

    fn node_to_head(&self) -> AtomicI128 {
        let tmpi64 = self.nref.load(Ordering::Acquire); // load从原子变量中读取当前值，指定读取操作的内存顺序为 Acquire
        let tmp_ptr = Arc::as_ptr(&self.next);
        let href = ((tmpi64 as i128) << 64) | (tmp_ptr as i128);
        AtomicI128::new(href)
    }

    pub(crate) fn deep_copy(&self) -> *mut Node<K, V> {
        unsafe {
            // 深拷贝 next 节点（递归拷贝）
            let next_ptr = self.next.load(Ordering::Acquire);
            let new_next = if !next_ptr.is_null() {
                // 非空时递归调用
                (*next_ptr).deep_copy()
            } else {
                std::ptr::null_mut()
            };
            // 创建新节点
            Node::new(self.key.clone(), self.value.clone(), new_next)
        }
    }
}

// Node struct representing an individual element in the linked list
pub(crate) struct Handle<K,V> {
    nref: AtomicI64,    // Atomic reference count (for lock-free decrement)
    next: AtomicPtr<Node<K, V>>, // Next node in the linked list
}

impl<K,V> Handle<K, V> {
    fn new(nref: i64, next: AtomicPtr<Node<K, V>>) -> Self{
        Handle{nref: AtomicI64::new(nref), next }
    }

    fn handle_to_head(&self) -> AtomicI128 {
        let tmpi64 = self.nref.load(Ordering::Acquire); // load从原子变量中读取当前值，指定读取操作的内存顺序为 Acquire
        let tmp_ptr = Arc::as_ptr(&self.next);
        let href = ((tmpi64 as i128) << 64) | (tmp_ptr as i128);
        AtomicI128::new(href)
    }
}

// Head struct holds the retirement list's head pointer and the active thread counter (HRef)
struct AtomicHead {
    href: AtomicI128, // Atomic I128 for lock-free updates of href and hptr together
}

impl<K,V> AtomicHead {
    fn new(href: AtomicI128) -> Self {
        AtomicHead {
            href: href, // Initialize HRef and HPtr to 0
        }
    }
    pub(crate) fn head_to_handle(&self) -> Arc<Handle<K, V>>{
        let tmpi128 = self.href.load(Ordering::Acquire);
        // let nref = AtomicI64::new((tmpi128 >> 64) as i64);
        let hptr = tmpi128 as usize;
        let ptr = AtomicPtr::new(hptr as *mut Node<K,V>);
        Arc::new(Handle::new((tmpi128 >> 64) as i64, ptr))
    }
}

// Handle struct to store a snapshot of HPtr for each thread
// #[derive(Clone, Copy)]
// pub(crate) struct Handle {
//     href_snapshot: i64,
//     hptr_snapshot: usize, // hptr serialized as usize
// }
//
// impl Handle {
//     fn to_i128(self) -> i128 {
//         ((self.href_snapshot as i128) << 64) | (self.hptr_snapshot as i128)
//     }
//
//     fn from_i128(value: i128) -> Self {
//         let href = (value >> 64) as i64;
//         let hptr = value as usize;
//         Handle { href_snapshot: href, hptr_snapshot: hptr }
//     }
// }

// MemoryTracker holds the global retirement list and provides methods for interacting with it
pub(crate) struct MemoryTracker<K,V> {
    head: AtomicHead, // Arc to share the Head structure across threads
    num_threads: i32,
    layout: Layout,
    // retired: AtomicU64,
}

impl<K,V> MemoryTracker<K,V> {
    // Create a new MemoryTracker instance
    pub(crate) fn new(num_threads: i32) -> Self {
        let layout = Layout::new::<Node<K, V>>();
        let mut head = AtomicHead::new(AtomicI128::new(0));
        MemoryTracker {
            head: AtomicHead::new(AtomicI128::new(0)),
            num_threads,
            layout: layout,
            //retired: AtomicU64::new(0),
        }
    }

    // Atomically increment HRef and return a snapshot of HPtr
    pub(crate) fn enter(&self) -> Arc<Handle<K,V>> {
        loop {
            // Get the current value of HRef and HPtr atomically
            // let current = self.head.href.load(Ordering::Acquire);
            // let href = self.head.nref.load(Ordering::Acquire);
            // let hptr = self.head.next.load(Ordering::Acquire);
            let current = self.head.href.load(Ordering::Acquire);
            let handle = self.head.head_to_handle();
            let href = handle.nref.load(Ordering::Acquire);
            let hptr = handle.next.load(Ordering::Acquire);

            // Create a snapshot handle

            // Try to atomically increment HRef and update HPtr with the same value
            let new_href = href + 1;
            let new_value= ((new_href as i128) << 64) | (hptr as i128);

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
    // pub(crate) fn get_retired_cnt(&self, tid: i32) -> i64 {
    //     if self.num_threads > 0 {
    //         self.retired.load(Ordering::Relaxed) as i64 / self.num_threads as i64
    //     } else {
    //         0
    //     }
    // }

    // Leave operation: decrement HRef and clean up any nodes if necessary
    pub(crate) fn leave(&self, handle: usize) { //Option<Arc<Node>>
        let mut tmp = 0 as usize;
        loop {
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;
            tmp = hptr;
            // if hptr != handle {
            //     //let mut next = AtomicPtr::new(hptr as *mut Node);
            //     let next_node = unsafe { Arc::from_raw(hptr as *const Node<K,V>) };
            //     let next = next_node.next.load(std::sync::atomic::Ordering::SeqCst);
            // }
            // If no threads are left, set HPtr to None (or null equivalent)
            let new_hptr = if href == 1 { 0 } else { hptr };

            // Decrement HRef to indicate thread leaving
            let new_href = href - 1;

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

        let mut prev = &tmp;
        let mut cur = prev.load(Ordering::SeqCst);
        while !cur==handle {
            unsafe {
                let cur_node = &*cur;
                let next = cur_node.next.load(Ordering::SeqCst);
                cur_node.nref.fetch_sub(1, Ordering::Release);
                if cur_node.nref == 0 {
                    if prev.compare_exchange(cur, next, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                        let value = cur_node.value.clone();
                        self.dealloc(cur, self.layout);
                        //Node::dealloc(cur);
                        //self.tracker.dealloc(cur_node as *mut u8, self.layout); // If the exchange fails, deallocate the node
                        return Some(value);
                    }
                }
                prev = &cur_node.next;
                cur = cur_node.next.load(Ordering::SeqCst);
            }
        }
    }

    // Retire operation: Insert new nodes into the linked list
    // fn retire(&self, new_node: Arc<Node<K,V>>) {
    //     loop {
    //         let current = self.head.href.load(Ordering::Acquire);
    //         let href = (current >> 64) as i64;
    //         let hptr = current as usize;
    //         //深拷贝new_node
    //         let new_head_node = new_node.deep_copy();
    //         //将new_head_node.next改为指向new_node，new_head_node.nref改为href
    //         new_head_node.store(new_node, Ordering::Release);
    //         new_head_node.nref.store(href, Ordering::Release);
    //         let new_head = new_head_node.node_to_head();
    //         //将new_node.next改为hptr
    //         new_node.next.store(hptr as *mut Node<K, V>, Ordering::Release);
    //         // Update the linked list
    //         match self.head.href.compare_exchange(
    //             current,
    //             new_head.load(Ordering::SeqCst),
    //             Ordering::Release,
    //             Ordering::Acquire,
    //         ) {
    //             Ok(_) => break,
    //             Err(_) => {}
    //         }
    //     }
        //将new_node.nref加上self.head.href的值
    //}
    pub(crate) fn retire<K, V>(&self, new_node: Arc<Node<K, V>>) {
        loop {
            // 获取当前链表头的值
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;

            // 深拷贝 new_node
            let new_head_node = unsafe { new_node.deep_copy() };

            // 将 new_head_node.next 改为指向 new_node，new_head_node.nref 改为 href
            unsafe {
                (*new_head_node).next.store(new_node.as_ptr(), Ordering::Release);
                (*new_head_node).nref.store(href, Ordering::Release);
            }

            // 生成新的链表头表示
            let new_head = unsafe { (*new_head_node).node_to_head() };

            // 将 new_node.next 改为指向 hptr
            new_node.next.store(hptr as *mut Node<K, V>, Ordering::Release);

            // 尝试使用 CAS 更新链表头
            match self.head.href.compare_exchange(
                current,
                new_head.load(Ordering::SeqCst),
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break, // 成功更新链表头，退出循环
                Err(_) => {} // 如果 CAS 失败，继续尝试
            }
        }

        // 将 new_node.nref 加上 self.head.href 的值
        let current_head_href = self.head.href.load(Ordering::Acquire);
        let current_nref = new_node.nref.load(Ordering::Acquire);
        let updated_nref = current_nref + (current_head_href >> 64) as i64;
        new_node.nref.store(updated_nref, Ordering::Release);
    }

    // Traverse the linked list and apply a function to each node
    // pub(crate) fn traverse(&self, handle: usize)
    // {
    //     // Atomically load the head pointer (HPtr) from the global Head structure
    //     let head_node = self.head.head_to_();
    //
    //
    //     // If the head pointer is null, the list is empty
    //     if hptr == 0 {
    //         return;
    //     }
    //
    //     // Start traversal from the first node
    //     let mut current = unsafe { Arc::from_raw(hptr as *const Node) };
    //
    //     while let Some(node) = Arc::get_mut(&mut current) {
    //         // Apply the user-provided function to the current node
    //         //func(node);
    //         node.nref.store(node.nref.load(Ordering::Acquire), Ordering::Release);
    //         // Move to the next node in the list
    //         if let Some(next) = &node.next {
    //             current = next.clone(); // Move to the next node
    //         } else {
    //             break; // End of the list
    //         }
    //     }
    // }
}

