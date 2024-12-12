// RAllocator trait extending Rideable
trait RAllocator: Rideable {
    // Allocates a block of memory. Returns a pointer (raw pointer in Rust)
    // tid: Thread ID, unique across all threads
    fn alloc_block(&self, tid: i32) -> *mut std::ffi::c_void;

    // Frees the block of memory
    // tid: Thread ID, unique across all threads
    fn free_block(&self, ptr: *mut std::ffi::c_void, tid: i32);
}

// Example concrete type that implements RAllocator
struct MemoryAllocator;

impl Rideable for MemoryAllocator {
    fn ride(&self) {
        println!("Memory Allocator riding!");
    }
}

impl RAllocator for MemoryAllocator {
    fn alloc_block(&self, tid: i32) -> *mut std::ffi::c_void {
        println!("Allocating block for thread {}", tid);
        // Simulate memory allocation, return a raw pointer (dummy pointer for illustration)
        let block: *mut i32 = Box::into_raw(Box::new(0));
        block as *mut std::ffi::c_void
    }

    fn free_block(&self, ptr: *mut std::ffi::c_void, tid: i32) {
        println!("Freeing block for thread {}", tid);
        // Convert the raw pointer back and free the allocated memory
        if !ptr.is_null() {
            unsafe {
                Box::from_raw(ptr as *mut i32); // Deallocates the memory when the Box is dropped
            }
        }
    }
}
