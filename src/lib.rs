use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::Path,
    sync::{Arc, Mutex, mpsc},
    thread,
};

#[derive(Deserialize, Debug)]
pub struct Config {
    #[allow(dead_code)]
    server: Option<Server>,
    #[allow(dead_code)]
    resources: Option<Vec<Resource>>,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    #[allow(dead_code)]
    host: Option<String>,
    #[allow(dead_code)]
    port: Option<String>,
    #[allow(dead_code)]
    threads: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct Resource {
    #[allow(dead_code)]
    request: String,
    #[allow(dead_code)]
    response: String,
}

impl Config {
    pub fn from_default_config_file() -> Result<Config, Box<dyn Error>> {
        //let etc_path = "/etc/rustweb/conf.toml";
        let etc_path = "/home/MattWaX/exercise-code/rust/rswebserver/conf.toml";
        if fs::exists(etc_path)? {
            let toml_str = fs::read_to_string(Path::new(&etc_path))?;
            let config: Config = toml::from_str(&toml_str)?;

            Ok(config)
        } else {
            todo!("Create a new config file if not existant");
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

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

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Send the job to a worker in the `ThreadPool`
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);

            worker.thread.join().unwrap();
        }
    }
}

pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    /// Create a new worker that listen for new jobs to execute
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected! Shutting down!");
                        break;
                    }
                }
            }
        });

        Worker { id, thread }
    }
}
