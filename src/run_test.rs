use libc::{sysconf, _SC_PAGESIZE};
use std::alloc::{alloc, dealloc, Layout};
// use std::fs::File;
// use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use rand::Rng;

struct ThreadInfo {
    thread_id: usize,
    seed: u64,
}

pub struct GlobalTest {
    warmup: u32,
    num_threads: usize,
}

fn thread_main(thread_info: ThreadInfo) {
    // 模拟线程的任务
    println!("Thread PID {:?} with seed {:?}", thread_info.thread_id, thread_info.seed);

    // 在这里可以执行具体的任务...
}

impl GlobalTest {
    pub(crate) fn new(warmup: u32, num_threads: usize) -> Self {
        Self { warmup, num_threads}
    }

    fn warm_memory(&self, megabytes: u32) -> i32 {
        // 计算需要预热的内存大小
        let preheat: usize = (megabytes as usize) * (2 << 20);
        let mut ret = 0;

        // 获取系统页大小
        let block_size = unsafe {sysconf(_SC_PAGESIZE)};
        let block_size = block_size as usize;

        // 计算需要分配的块数
        let to_alloc = preheat / block_size;

        // 存储已分配的内存块
        // Vec::new()：创建一个空列表，创建一个空列表时需要指明数据类型
        // 当需要写入内存时，必须用mut修饰，代表可变变量
        let mut allocd: Vec<*mut u8> = Vec::new();
        // layout指定的大小和对齐方式
        // alloc按照layout指定的方式分配内存
        let layout = Layout::array::<u8>(block_size / 4).expect("Invalid layout");
        // 分配内存
        for _ in 0..to_alloc {
            // 尝试分配块
            unsafe {
                // 分配内存，同c++中的 int32_t* ptr  = (int32_t*)malloc(blockSize);
                let ptr = alloc(layout);
                let ptr2 = alloc(layout);
                let ptr3 = alloc(layout);

                if ptr.is_null() || ptr2.is_null() || ptr3.is_null() {
                    // 如果内存分配失败，那么warmup返回-1
                    if !ptr.is_null() { dealloc(ptr, layout); }
                    if !ptr2.is_null() { dealloc(ptr2, layout); }
                    if !ptr3.is_null() { dealloc(ptr3, layout); }
                    ret = -1;
                    break;
                }
                // 释放之前分配的内存
                dealloc(ptr2, layout);
                dealloc(ptr3, layout);
                ptr.write(1);
                allocd.push(ptr);
            }
        }
        // 释放内存
        unsafe {
            for &p in &allocd {
                dealloc(p, layout);
            }
        }
        return ret;
    }

    pub(crate) fn run_test(&self) {
        if self.warmup > 0 {
            let error_code = self.warm_memory(self.warmup);
            println!("Error in warmup! End with code {:?}", error_code);
        }

        self.parallel_work();

        // if let Some(out_file) = &self.out_file {
        //     if let Some(recorder) = &self.recorder {
        //         let mut file = File::create(out_file).expect("Failed to create output file");
        //         writeln!(file, "{}", recorder.get_csv()).expect("Failed to write output");
        //     }
        // }
    }

    fn parallel_work(&self) {
        let mut handles = vec![];

        for i in 0..self.num_threads {
            // 为每个线程分配一个 pid 和随机种子
            let pid = i;  // 线程ID作为pid
            let seed: u64 = rand::thread_rng().random();  // 随机生成一个种子

            let thread_info = ThreadInfo { thread_id:pid, seed };

            // 创建线程并将 thread_info 移动到线程
            let handle = thread::spawn(move || {
                thread_main(thread_info);  // 每个线程执行独立的 `thread_main` 函数
            });

            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
