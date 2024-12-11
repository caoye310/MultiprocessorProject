use libc::{sysconf, _SC_PAGESIZE};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::File;
use std::sync::{Arc, Mutex};
use sysinfo::{System, SystemExt};
use std::time::Duration;
use std::io::Write;
use std::thread;
use std::time::{Instant};
use rand::Rng;
use crate::run_test::link_list::SortedUnorderedMap;
extern crate libc;
use sys_info;
mod link_list;

struct ThreadInfo {
    thread_id: i32,
    //seed: u64,
}

pub struct GlobalTest {
    warmup: u32,
    num_threads: i32,
    contain_percent:f64,
    list: SortedUnorderedMap<i64, i32>,
    //queue = Arc::new(Mutex::new(Vec::new()));
}

impl GlobalTest {
    pub(crate) fn new(warmup: u32, num_threads: i32, contain_percent:f64) -> Self {
        let list = SortedUnorderedMap::new(1, num_threads);
        GlobalTest { warmup, num_threads, contain_percent, list}
    }

    fn thread_main_debug(&self, contain_percent: f64) {
        let mut rng = rand::thread_rng();
       // println!("Thread PID {:?} with seed {:?}", thread_info.thread_id, thread_info.seed);
        for i in 1..3000 {
            let random_float: f64 = rng.gen_range(0.0..1.0);
            let random_int: i64 = rng.gen_range(0..5);
            if random_float < (1.0 - contain_percent) / 2.0 {
                //println!("Insert key {:?}", random_int);
                if self.list.insert(random_int, i, 0) {
                    //println!("Insert key {:?} success", random_int);
                }else{
                    //println!("Insert key {:?} failed: duplicate", random_int);
                }
                continue;
            }
            if random_float >= (1.0 - contain_percent) / 2.0 && random_float < (1.0 - contain_percent) {
                //println!("Remove {:?}", random_int);
                self.list.remove(&random_int, 0);
                //println!("Remove key: {:?}", self.list.remove(&random_int, 0));
                continue;
            }
            //println!("Get key {:?}", random_int);
            //println!("Get key : {:?}", self.list.get(&random_int, 0));
            self.list.get(&random_int, 0);
        }
    }

    fn thread_main(&self, thread_info: ThreadInfo, contain_percent: f64) {
        let mut rng = rand::thread_rng();
        //println!("Thread PID {:?} with seed {:?}", thread_info.thread_id, thread_info.seed);
        for i in 1..50000 {
            let random_float: f64 = rng.gen_range(0.0..1.0);
            let random_int: i64 = rng.gen_range(0..100);
            if random_float < (1.0 - contain_percent) / 2.0 {
                //println!("Insert key {:?}", random_int);
                if self.list.insert(random_int, i, thread_info.thread_id) {
                    //println!("Insert key {:?} success", random_int);
                }else{
                    //println!("Insert key {:?} failed: duplicate", random_int);
                }
                continue;
            }
            if random_float >= (1.0 - contain_percent) / 2.0 && random_float < (1.0 - contain_percent) {
                //println!("Remove {:?}", random_int);
                //println!("Remove key: {:?}", self.list.remove(&random_int, thread_info.thread_id));
                self.list.remove(&random_int, thread_info.thread_id);
                continue;
            }
            //println!("Get key {:?}", random_int);
            //println!("Get key : {:?}", self.list.get(&random_int, thread_info.thread_id));
            self.list.get(&random_int, thread_info.thread_id);
        }
    }

    fn warm_memory(&self, megabytes: u32) -> i32 {
        // The size of memory
        let preheat: usize = (megabytes as usize) * (2 << 20);
        let mut ret = 0;

        // Get the size of system page
        let block_size = unsafe { sysconf(_SC_PAGESIZE) };
        let block_size = block_size as usize;

        // Calculate how many blocks we need
        let to_alloc = preheat / block_size;

        // Vec::new()：创建一个空列表，创建一个空列表时需要指明数据类型
        // we need mut if we want to write sth to the memory, mut means the variable is changeable
        let mut allocd: Vec<*mut u8> = Vec::new();
        // layout: memory size and align method
        // Alloc function will alloc memory according to the layout
        let layout = Layout::array::<u8>(block_size / 4).expect("Invalid layout");
        for _ in 0..to_alloc {
            unsafe {
                // alloc memory, similar to int32_t* ptr  = (int32_t*)malloc(blockSize); in c++
                let ptr = alloc(layout);
                let ptr2 = alloc(layout);
                let ptr3 = alloc(layout);

                if ptr.is_null() || ptr2.is_null() || ptr3.is_null() {
                    // If alloc failed，return -1
                    if !ptr.is_null() { dealloc(ptr, layout); }
                    if !ptr2.is_null() { dealloc(ptr2, layout); }
                    if !ptr3.is_null() { dealloc(ptr3, layout); }
                    ret = -1;
                    break;
                }
                // Release the memory
                dealloc(ptr2, layout);
                dealloc(ptr3, layout);
                ptr.write(1);
                allocd.push(ptr);
            }
        }
        // Dealloc the memory
        unsafe {
            for &p in &allocd {
                dealloc(p, layout);
            }
        }
        return ret;
    }

    pub(crate) fn run_test(self) {
        if self.warmup > 0 {
            let error_code = self.warm_memory(self.warmup);
            if error_code != 0 {
                println!("Error in warmup! End with code {:?}", error_code);
            }
        }
        let debug=false;
        if !debug{
            let mut handles = vec![];
            let start = Instant::now();
            let number_of_threads = self.num_threads;
            let contain_percent = self.contain_percent;
            // Wrap self in an Arc and Mutex for safe shared ownership
            let self_arc = Arc::new(Mutex::new(self));  // Wrap `self` in an Arc<Mutex<YourStruct>>

            // 创建一个线程安全的共享变量存储内存数据
            let memory_data = Arc::new(Mutex::new(Vec::new()));
            let system_arc = Arc::new(Mutex::new(System::new_all()));

            // 定时任务线程
            let memory_data_clone = Arc::clone(&memory_data);
            let system_clone = Arc::clone(&system_arc);
            let monitor_handle = thread::spawn(move || {
                for _ in 0..(35.0/0.02) as usize { // 1秒钟的采样次数，(1.0 / 0.001) 代表每秒 1000 次
                    {
                        // 锁定 system_clone 并更新内存信息
                        let mut system = system_clone.lock().unwrap();
                        system.refresh_memory(); // 假设你有一个 `refresh_memory` 方法

                        // Get system's available memory
                        match sys_info::mem_info() {
                            Ok(info) => {
                                // 输出可用内存，以字节为单位
                                //let mut available_memory= ;
                                let mut data = memory_data_clone.lock().unwrap();
                                data.push(info.avail);
                                //println!("Available memory: {} bytes", info.avail);
                            },
                            Err(e) => eprintln!("Error fetching memory info: {}", e)
                        }

                    }
                    // 每隔 1 毫秒睡眠
                    thread::sleep(Duration::from_millis(20));
                }
            });

            for i in 0..number_of_threads {
                let pid = i;  // thread id
                //let seed: u64 = rand::thread_rng().random();

                let thread_info = ThreadInfo { thread_id: pid};
                let self_clone = Arc::clone(&self_arc); // Clone the Arc to share ownership across threads
                // Create a thread and give it thread_info
                let handle = thread::spawn(move || {
                    let self_locked = self_clone.lock().unwrap();
                    self_locked.thread_main(thread_info, contain_percent);  // Each thread run `thread_main` independently
                });

                handles.push(handle);
            }

            // Waiting for all the threads finish
            for handle in handles {
                handle.join().unwrap();
            }
            monitor_handle.join().unwrap();

            let duration = start.elapsed().as_nanos();
            println!("Execution time: {:?} nanosecond", duration);

            // 输出内存数据到 CSV 文件
            let data_file = "memory_data.csv";
            let mut file = File::create(data_file).expect("file creation error");
            let memory_data = memory_data.lock().unwrap();
            for (i, &value) in memory_data.iter().enumerate() {
                writeln!(file, "{},{:.2}", i as f64 * 0.2, value as i64).expect("error writing memory data");
            }

        }
        else{
            self.thread_main_debug(0.2);
        }
    }
}