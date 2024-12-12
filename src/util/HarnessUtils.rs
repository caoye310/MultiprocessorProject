use std::env;
use std::ffi::CString;
use std::process;
use std::ptr;
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;
use libc::{gethostname, sysconf, _SC_PAGESIZE};

fn errexit(err_str: &str) {
    eprintln!("{}", err_str);
    process::exit(1);
}

extern crate backtrace;
use backtrace::Backtrace;

fn fault_handler(sig: i32) {
    let backtrace = Backtrace::new();
    eprintln!("Error: signal {}:", sig);
    eprintln!("{:?}", backtrace);
    process::exit(1);
}

fn is_integer(s: &str) -> bool {
    // check for empty case, and easy fail at start of string
    if s.is_empty() || (!s.starts_with('-') && !s.starts_with('+') && !s.chars().next().unwrap().is_digit(10)) {
        return false;
    }

    // Rust does not have strtol directly, so we will use `i64::from_str_radix`
    s.parse::<i64>().is_ok()
}

fn machine_name() -> String {
    let mut hostname = vec![0u8; 1024];
    unsafe {
        gethostname(hostname.as_mut_ptr() as *mut i8, 1024);
    }
    String::from_utf8_lossy(&hostname).to_string()
}

fn arch_bits() -> Option<i32> {
    if std::mem::size_of::<*const ()>() == 8 {
        Some(64)
    } else if std::mem::size_of::<*const ()>() == 16 {
        Some(128)
    } else if std::mem::size_of::<*const ()>() == 4 {
        Some(32)
    } else if std::mem::size_of::<*const ()>() == 2 {
        Some(8)
    } else {
        None
    }
}

fn next_rand(last: u32) -> u32 {
    let mut next = last;
    next = next * 1664525 + 1013904223;
    next
}

fn warm_memory(megabytes: u32) -> i32 {
    let preheat = (megabytes as u64) * (2 << 20);
    let block_size = unsafe { sysconf(_SC_PAGESIZE) as u64 };
    let to_alloc = (preheat / block_size) as usize;

    let mut allocd: VecDeque<*mut i32> = VecDeque::new();
    let mut ret = 0;

    for _ in 0..to_alloc {
        unsafe {
            let ptr = libc::malloc(block_size as usize) as *mut i32;
            let ptr2 = libc::malloc(block_size as usize) as *mut i32;
            let ptr3 = libc::malloc(block_size as usize) as *mut i32;

            if ptr.is_null() || ptr2.is_null() || ptr3.is_null() {
                ret = -1;
                break;
            }

            // Simulate some work done with the allocated memory
            *ptr = 1;

            libc::free(ptr2 as *mut libc::c_void);
            libc::free(ptr3 as *mut libc::c_void);
            allocd.push_back(ptr);
        }
    }

    for _ in 0..to_alloc {
        unsafe {
            if let Some(ptr) = allocd.pop_back() {
                libc::free(ptr as *mut libc::c_void);
            }
        }
    }

    ret
}