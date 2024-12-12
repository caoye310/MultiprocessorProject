use std::mem::{size_of, align_of};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// Define CACHE_LINE_SIZE based on the LEVEL1_DCACHE_LINESIZE constant.
const CACHE_LINE_SIZE: usize = 128;

// Generic padded struct to align data to the cacheline size.
#[repr(C, align(128))]
pub struct Padded<T> {
    data: T,
    pad: [u8; CACHE_LINE_SIZE - (size_of::<T>() % CACHE_LINE_SIZE)],
}

impl<T> Padded<T> {
    // Default constructor.
    pub fn new() -> Self
    where
        T: Default,
    {
        Self {
            data: T::default(),
            pad: [0; CACHE_LINE_SIZE - (size_of::<T>() % CACHE_LINE_SIZE)],
        }
    }

    // Constructor with a value.
    pub fn from_value(value: T) -> Self {
        Self {
            data: value,
            pad: [0; CACHE_LINE_SIZE - (size_of::<T>() % CACHE_LINE_SIZE)],
        }
    }
}

// Implement Deref and DerefMut for convenient access to `data`.
impl<T> Deref for Padded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Padded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

// Implement conversion from T to Padded<T>.
impl<T> From<T> for Padded<T> {
    fn from(value: T) -> Self {
        Self::from_value(value)
    }
}

// Implement conversion from Padded<T> to T.
impl<T: Clone> From<Padded<T>> for T {
    fn from(padded: Padded<T>) -> Self {
        padded.data.clone()
    }
}


pub struct PaddedAtomic<T> {
    ui: T,
    pad: [u8; CACHE_LINE_SIZE - (size_of::<T>() % CACHE_LINE_SIZE)],
}

impl PaddedAtomic<AtomicUsize> {
    // Default constructor
    pub fn new() -> Self {
        Self {
            ui: AtomicUsize::new(0),
            pad: [0; CACHE_LINE_SIZE - (size_of::<AtomicUsize>() % CACHE_LINE_SIZE)],
        }
    }

    // Constructor with a value
    pub fn from_value(value: usize) -> Self {
        Self {
            ui: AtomicUsize::new(value),
            pad: [0; CACHE_LINE_SIZE - (size_of::<AtomicUsize>() % CACHE_LINE_SIZE)],
        }
    }

    // Load operation
    pub fn load(&self, order: Ordering) -> usize {
        self.ui.load(order)
    }

    // Store operation
    pub fn store(&self, value: usize, order: Ordering) {
        self.ui.store(value, order)
    }
}

// Conversion from PaddedAtomic<AtomicUsize> to usize
impl From<PaddedAtomic<AtomicUsize>> for usize {
    fn from(padded: PaddedAtomic<AtomicUsize>) -> Self {
        padded.ui.load(Ordering::SeqCst)
    }
}


pub struct VolatilePadded<T> {
    ui: T,
    pad: [u8; if CACHE_LINE_SIZE > size_of::<T>() {
        CACHE_LINE_SIZE - size_of::<T>()
    } else {
        1
    }],
}

impl<T: Default> VolatilePadded<T> {
    // Default constructor
    pub fn new() -> Self {
        Self {
            ui: T::default(),
            pad: [0; if CACHE_LINE_SIZE > size_of::<T>() {
                CACHE_LINE_SIZE - size_of::<T>()
            } else {
                1
            }],
        }
    }
}

impl<T> VolatilePadded<T> {
    // Constructor with a value
    pub fn from_value(val: T) -> Self {
        Self {
            ui: val,
            pad: [0; if CACHE_LINE_SIZE > size_of::<T>() {
                CACHE_LINE_SIZE - size_of::<T>()
            } else {
                1
            }],
        }
    }

    // Read the value using volatile semantics
    pub fn load(&self) -> T
    where
        T: Copy,
    {
        unsafe { std::ptr::read_volatile(&self.ui) }
    }

    // Write a value using volatile semantics
    pub fn store(&mut self, val: T) {
        unsafe { std::ptr::write_volatile(&mut self.ui, val) }
    }
}

impl<T> From<T> for VolatilePadded<T> {
    // Conversion from T to VolatilePadded<T>
    fn from(val: T) -> Self {
        Self::from_value(val)
    }
}

impl<T: Copy> From<VolatilePadded<T>> for T {
    // Conversion from VolatilePadded<T> to T
    fn from(padded: VolatilePadded<T>) -> Self {
        padded.load()
    }
}

pub struct CPtrLocal<T> {
    ui: u64,
}

impl<T> CPtrLocal<T> {
    // Initialize with a pointer and a sequence number.
    pub fn init(&mut self, ptr: *const T, sn: u32) {
        let mut value = 0u64;
        value |= (ptr as u64) << 32; // Store the pointer in the high 32 bits.
        value |= sn as u64;         // Store the sequence number in the low 32 bits.
        self.ui = value;
    }

    // Initialize with a raw 64-bit value.
    pub fn init_raw(&mut self, value: u64) {
        self.ui = value;
    }

    // Retrieve the entire 64-bit representation.
    pub fn all(&self) -> u64 {
        self.ui
    }

    // Retrieve the pointer portion (high 32 bits).
    pub fn ptr(&self) -> *const T {
        ((self.ui & 0xFFFFFFFF00000000) >> 32) as *const T
    }

    // Retrieve the sequence number (low 32 bits).
    pub fn sn(&self) -> u32 {
        (self.ui & 0x00000000FFFFFFFF) as u32
    }

    // Store a null pointer.
    pub fn store_null(&mut self) {
        self.ui = 0;
    }

    // Dereference operator.
    pub unsafe fn deref(&self) -> &T {
        &*self.ptr()
    }

    // Constructor: default (null pointer and sequence number 0).
    pub fn new() -> Self {
        Self { ui: 0 }
    }

    // Constructor: from raw 64-bit value.
    pub fn from_raw(value: u64) -> Self {
        let mut instance = Self::new();
        instance.init_raw(value);
        instance
    }

    // Constructor: from pointer and sequence number.
    pub fn from_ptr_sn(ptr: *const T, sn: u32) -> Self {
        let mut instance = Self::new();
        instance.init(ptr, sn);
        instance
    }
}

// Implementations for assignment and conversions.
impl<T> From<*const T> for CPtrLocal<T> {
    fn from(ptr: *const T) -> Self {
        Self::from_ptr_sn(ptr, 0)
    }
}

impl<T> From<u64> for CPtrLocal<T> {
    fn from(value: u64) -> Self {
        Self::from_raw(value)
    }
}

impl<T> From<CPtrLocal<T>> for u64 {
    fn from(cptr: CPtrLocal<T>) -> Self {
        cptr.all()
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
use std::ptr;

#[derive(Debug)]
pub struct CPtr<T> {
    ui: AtomicU64,
}

impl<T> CPtr<T> {
    /// Initializes the atomic value with a pointer and sequence number.
    pub fn init(&self, ptr: *const T, sn: u32) {
        let mut value = 0u64;
        value |= (ptr as u64) << 32; // Store the pointer in the high 32 bits.
        value |= sn as u64;         // Store the sequence number in the low 32 bits.
        self.ui.store(value, Ordering::Release);
    }

    /// Initializes the atomic value with a raw 64-bit integer.
    pub fn init_raw(&self, value: u64) {
        self.ui.store(value, Ordering::Release);
    }

    /// Dereference the stored pointer.
    pub unsafe fn deref(&self) -> &T {
        &*self.ptr()
    }

    /// Returns the pointer portion of the atomic value.
    pub fn ptr(&self) -> *const T {
        let raw = self.ui.load(Ordering::Acquire);
        ((raw & 0xFFFFFFFF00000000) >> 32) as *const T
    }

    /// Returns the sequence number portion of the atomic value.
    pub fn sn(&self) -> u32 {
        (self.ui.load(Ordering::Acquire) & 0x00000000FFFFFFFF) as u32
    }

    /// Returns the full 64-bit atomic value.
    pub fn all(&self) -> u64 {
        self.ui.load(Ordering::Acquire)
    }

    /// Compare-and-swap operation with a pointer and incremented sequence number.
    pub fn cas(
        &self,
        oldval: &CPtrLocal<T>,
        newval: *const T,
    ) -> bool {
        let mut replacement = CPtrLocal::new();
        replacement.init(newval, oldval.sn() + 1);
        let old = oldval.all();
        self.ui
            .compare_exchange_strong(old, replacement.all(), Ordering::Release, Ordering::Relaxed)
            .is_ok()
    }

    /// Compare-and-swap operation with two local counted pointers.
    pub fn cas_local(
        &self,
        oldval: &CPtrLocal<T>,
        newval: &CPtrLocal<T>,
    ) -> bool {
        let mut replacement = CPtrLocal::new();
        replacement.init(newval.ptr(), oldval.sn() + 1);
        let old = oldval.all();
        self.ui
            .compare_exchange_strong(old, replacement.all(), Ordering::Release, Ordering::Relaxed)
            .is_ok()
    }

    /// Store a null pointer.
    pub fn store_null(&self) {
        self.init(ptr::null(), 0);
    }

    /// Repeatedly attempts to store a pointer until success.
    pub fn store_ptr(&self, newval: *const T) {
        loop {
            let oldval = CPtrLocal::from_raw(self.all());
            if self.cas(&oldval, newval) {
                break;
            }
        }
    }

    /// Creates a new counted pointer initialized to null.
    pub fn new() -> Self {
        let instance = Self {
            ui: AtomicU64::new(0),
        };
        instance
    }

    /// Creates a new counted pointer with a raw 64-bit value.
    pub fn from_raw(value: u64) -> Self {
        let instance = Self::new();
        instance.init_raw(value);
        instance
    }

    /// Creates a new counted pointer with a pointer and sequence number.
    pub fn from_ptr_sn(ptr: *const T, sn: u32) -> Self {
        let instance = Self::new();
        instance.init(ptr, sn);
        instance
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CPtrLocal<T> {
    ui: u64,
}

impl<T> CPtrLocal<T> {
    pub fn new() -> Self {
        Self { ui: 0 }
    }

    pub fn init(&mut self, ptr: *const T, sn: u32) {
        let mut value = 0u64;
        value |= (ptr as u64) << 32;
        value |= sn as u64;
        self.ui = value;
    }

    pub fn init_raw(&mut self, value: u64) {
        self.ui = value;
    }

    pub fn ptr(&self) -> *const T {
        ((self.ui & 0xFFFFFFFF00000000) >> 32) as *const T
    }

    pub fn sn(&self) -> u32 {
        (self.ui & 0x00000000FFFFFFFF) as u32
    }

    pub fn all(&self) -> u64 {
        self.ui
    }

    pub fn from_raw(value: u64) -> Self {
        let mut instance = Self::new();
        instance.init_raw(value);
        instance
    }
}
