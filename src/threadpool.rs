use std::sync::mpsc;
use std::sync::{Arc, Mutex};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

struct Worker {
    thread: std::thread::JoinHandle<()>,
    id: i32
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(i: i32) -> ThreadPool{
        let (tx, rx) = mpsc::channel::<Job>();
        let rx_wrapped = Arc::new(Mutex::new(rx));
        let mut workers = Vec::new();
        for id in 0..i {
            let cloned = rx_wrapped.clone();
            let thread = std::thread::spawn( move || {
                let own_rx = cloned;
                while let Ok(job) = own_rx.lock().unwrap().recv() {
                    job()
                }

            });
            workers.push(Worker{
                thread,
                id
            });
        }
        ThreadPool{
            workers,
            sender: tx,
        }
    }
    pub fn execute<F: FnOnce() + Send + 'static >(&self, f: F) {
        let boxed = Box::new(f);
        let _ = &self.sender.send(boxed).unwrap();
    }
}

