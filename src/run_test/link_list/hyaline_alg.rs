use std::sync::atomic::{AtomicI64, AtomicPtr, Ordering};
use portable_atomic::AtomicI128;
use std::sync::Arc;
use std::alloc::{Layout, alloc, dealloc};
use std::fmt::Debug;

pub(crate) struct MyAlloc {}
// Alloc/Dealloc memory and return a raw pointer to an object
impl MyAlloc {
    pub(crate) fn new() -> MyAlloc {
        MyAlloc {}
    }

    pub(crate) fn alloc<K,V>(&self, layout: Layout) -> *mut u8 {
        unsafe {
            // Alloc memory for node
            let ptr = alloc(layout);
            // If failed print "Memory allocation failed!"
            if ptr.is_null() {
                panic!("Memory allocation failed!");
            }
            ptr
        }
    }
    // dealloc memory
    pub(crate) fn dealloc<K,V>(&self, ptr: *mut u8, layout: Layout) {
        unsafe { dealloc(ptr, layout) };
    }
}


pub(crate) struct Node<K, V> {
    pub(crate) key: K,
    pub(crate) value: V,
    nref: AtomicI64,    // Atomic reference count (for lock-free decrement)
    pub(crate) next: AtomicPtr<Node<K, V>>,
}

impl<K, V> Node<K, V> {
    // create a new node
    pub(crate) fn new(key: K, value: V, next: *mut Node<K, V>) -> *mut Node<K, V> {
        unsafe {
            let layout = Layout::new::<Node<K, V>>();
            let mem_manage= MyAlloc::new();
            // Alloc memory for the new node
            let ptr = mem_manage.alloc::<K, V>(layout) as *mut Node<K, V>;
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

    // unsafe fn dealloc(ptr: *mut Node<K, V>) {
    //     let layout = Layout::new::<Node<K, V>>();
    //     let mem_manage = MyAlloc::new();
    //     mem_manage.dealloc::<K,V>(ptr as *mut u8, layout);
    // }
    // change node to be a head (because every time when a thread call entry,
    // we need to use compare_exchange to update the head. Thus, we need to
    // change the node to an atomic head first)
    // fn node_to_head(&self) -> AtomicI128 {
    //     let tmpi64 = self.nref.load(Ordering::Acquire);
    //     let tmp_ptr = self.next.load(Ordering::Acquire) as i128;
    //     let href = ((tmpi64 as i128) << 64) | tmp_ptr;
    //     AtomicI128::new(href)
    // }
}

// when a thread entries, we need to return snapshot of the head
pub(crate) struct Handle<K,V> {
    nref: AtomicI64,    // Atomic reference count (for lock-free decrement)
    next: AtomicPtr<Node<K, V>>, // Next node in the linked list
}

impl<K,V> Handle<K, V> {
    fn new(nref: i64, next: AtomicPtr<Node<K, V>>) -> Self{
        Handle{
            nref: AtomicI64::new(nref),
            next,
        }
    }
}

// Head struct holds the retirement list's head pointer and the active thread counter (HRef)
struct AtomicHead {
    href: AtomicI128, // Atomic I128 for lock-free updates of href and hptr together
}

impl AtomicHead {
    fn new(href: AtomicI128) -> Self {
        AtomicHead {
            href, // Initialize HRef and HPtr to 0
        }
    }
    // Before return, change head to handle
    pub(crate) fn head_to_handle<K, V>(&self) -> Arc<Handle<K, V>>{
        let tmpi128 = self.href.load(Ordering::Acquire);
        let nref = (tmpi128 >> 64) as i64;
        let hptr = tmpi128 as usize;
        let ptr = AtomicPtr::new(hptr as *mut Node<K,V>);
        Arc::new(Handle::new(nref, ptr))
    }
}

// MemoryTracker holds the global retirement list and provides methods for interacting with it
pub(crate) struct MemoryTracker {
    head: AtomicHead, // Arc to share the Head structure across threads
    //num_threads: i32,
    layout: Layout, // layout for the node
}

impl MemoryTracker
{
    // Create a new MemoryTracker instance
    pub(crate) fn new<K,V>() -> Self {
        let layout = Layout::new::<Node<K, V>>();
        MemoryTracker {
            head: AtomicHead::new(AtomicI128::new(0)),
            //num_threads,
            layout,
        }
    }

    pub(crate) fn print<K, V>(&self) where
        K: Clone + Debug,
        V: Clone + Debug,
    {
        println!("print");
        //let tmpi128 = self.head.href.load(Ordering::Acquire);
        let handle = self.head.head_to_handle();
       // let href = handle.nref.load(Ordering::Acquire);
        let node_ptr = handle.next.load(Ordering::Acquire);

        // Output string stored in result
        let mut result = String::new();
        result.push_str(&format!("({:?}, {:?})",
                                 handle.nref.load(Ordering::Acquire),
                                 handle.next.load(Ordering::Acquire)));
        let mut current_node = node_ptr;

        while !current_node.is_null() {
            unsafe {
                let node = &*current_node;  // 解引用当前节点
                result.push_str(&format!(" -> ({:?}, {:?}, {:?}, {:?})",
                                         node.key,
                                         node.value,
                                         node.nref.load(Ordering::Acquire),
                                         node.next.load(Ordering::Acquire)));
                // Get the pointer to the next node
                current_node = node.next.load(Ordering::Acquire) as *mut Node<K, V>;
            }
        }
        println!("{}", result);
    }

    // Atomically increment HRef and return a snapshot of HPtr
    pub(crate) fn enter<K,V>(&self) -> Arc<Handle<K,V>>  where
        K: Clone + Debug,
        V: Clone + Debug,
    {
        //println!("enter ...");
        loop {
            // Get the current head
            let current = self.head.href.load(Ordering::Acquire);
            // The snapshot handle of current list
            let handle = self.head.head_to_handle();
            let href = handle.nref.load(Ordering::Acquire);
            let hptr = handle.next.load(Ordering::Acquire);


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

    // Leave operation: decrement HRef and clean up any nodes if necessary
    pub(crate) fn leave<K,V>(&self, handle: &AtomicPtr<Handle<K,V>>)
    where
        K: Clone + Debug,
        V: Clone + Debug,
    { //Option<Arc<Node>>
        //println!("leave ...");
        //self.print::<K, V>();
        loop {
            // get the current head
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            // next node
            let hptr = current as usize;

            // Decrement HRef to indicate thread leaving
            let new_href = href - 1;

            // Update HRef and HPtr atomically
            let new_value = ((new_href as i128) << 64) | (hptr as i128);

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
        // Traverse the sublist from the current HPtr to the `handle` node
        unsafe {
            let mut current = if self.head.href.load(Ordering::Acquire) != 0 {
                let head_node = &*(self.head.href.load(Ordering::Acquire) as *mut Node<K, V>);
                head_node.next.load(Ordering::Acquire)
            } else {
                std::ptr::null_mut() // If head is null, no traversal is needed
            };
            let mut prev: *mut Node<K,V> = std::ptr::null_mut();//: *mut Node<K, V> = self.head.href.load(Ordering::Acquire) as *mut Node<K, V>; // Track the previous node
            while !current.is_null() {
                // Check if we've reached the handle node
                let current_node = &*current;
                // Decrement the NRef counter
                let prev_nref = current_node.nref.fetch_sub(1, Ordering::Release);

                // If NRef reaches 0, deallocate the node
                if prev_nref == 1 {
                    let next_node = current_node.next.load(Ordering::Acquire);
                    if !prev.is_null() {
                        let prev_node = &*prev;
                        prev_node
                            .next
                            .compare_exchange(current, next_node, Ordering::Release, Ordering::Relaxed)
                            .unwrap(); // CAS to update the next pointer
                    }else{
                        // If current is the next of head, don't forget to update the head! change the hptr of head to 0!
                        loop {
                            // the snapshot handle of current list
                            let up_head_current = self.head.href.load(Ordering::Acquire);
                            let up_head_handle = self.head.head_to_handle::<K, V>();
                            let up_head_href = up_head_handle.nref.load(Ordering::Acquire);
                            // Try to atomically increment HRef and update HPtr with the same value
                            let up_head_new_value= ((up_head_href as i128) << 64) | (next_node as i128);

                            // Perform a CAS operation to update the HRef and HPtr together using compare_exchange
                            match self.head.href.compare_exchange(
                                up_head_current,      // expected value
                                up_head_new_value,    // new value
                                Ordering::Release, // success ordering
                                Ordering::Acquire, // failure ordering
                            ) {
                                Ok(_) => break handle, // Successfully updated, return the snapshot
                                Err(_) => {} // CAS failed, retry
                            }
                        };
                    }
                    let mem_manage= MyAlloc::new();
                    mem_manage.dealloc::<K,V>(current as * mut u8, self.layout);
                    // If current is the next of head, don't forget to update the head! change the hptr of head to 0!
                }
                if current == handle.load(Ordering::Acquire) as * mut Node<K,V>{
                    break;
                }
                // Move to the next node
                prev = current;
                current = current_node.next.load(Ordering::Acquire);
            }
        }
        //self.print::<K, V>();
    }

    pub(crate) fn retire<K,V>(&self, new_node: Arc<Node<K, V>>)  where
        K: Clone + Debug,
        V: Clone + Debug,
    {
        //println!("retire ...");
        loop {
            // Get the head
            let current = self.head.href.load(Ordering::Acquire);
            let href = (current >> 64) as i64;
            let hptr = current as usize;
            // The head point to the new node so the new node is inserted after the head
            // new_node.next point to the node that the old head point to
            new_node.next.store(hptr as *mut Node<K, V>, Ordering::Release);

            let new_head= ((href as i128) << 64) | ((Arc::as_ptr(&new_node) as usize) as i128);

            // Use CAS to update the head
            match self.head.href.compare_exchange(
                current,
                new_head,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(_) => {} // If CAS failed, keep trying
            }
        }

        // "adjust" mentioned in the paper. new_node.nref add the value of self.head.href
        loop {
            let current_head_href = self.head.href.load(Ordering::Acquire);
            let current_nref = new_node.nref.load(Ordering::Acquire);
            let updated_nref = current_nref + (current_head_href >> 64) as i64;

            // Use CAS to atomically update `new_node.nref`.
            if new_node.nref.compare_exchange(
                current_nref,
                updated_nref,
                Ordering::Release,
                Ordering::Acquire,
            ).is_ok() {
                break; // Exit the loop if CAS succeeds
            }
        }
    }
}
