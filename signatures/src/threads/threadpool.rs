use std::{sync::{mpsc, Mutex, Arc}, thread};

use log::trace;

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, std::io::Error> {
        if size < 1 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Cannot create threadpool with no threads"));
        };
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Ok(ThreadPool {workers, sender})
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(job).unwrap()
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            while let Ok(job) = receiver.lock().unwrap().recv() {
                trace!("Worker {id} got job");
                job();
            }
        });
        Worker {id, thread}
    }
}