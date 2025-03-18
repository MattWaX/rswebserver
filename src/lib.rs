use std::thread;

pub struct ThreadPool {
    threads: Vec<Worker>,
}

impl ThreadPool {
    /// Create a ThreadPool
    ///
    /// It takes how many thread will be in the pool as an argument
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut threads = Vec::with_capacity(size);

        for i in 0..size {
            threads.push(Worker::new(i));
        }

        ThreadPool{ threads }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize) -> Worker {
        let thread = thread::spawn(||{});
        Worker {
            id,
            thread,
        }
    }
}
