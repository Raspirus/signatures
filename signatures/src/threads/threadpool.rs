use std::{sync::{mpsc, Mutex, Arc}, thread};

use log::{trace, info, debug};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    _workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, std::io::Error> {
        if size < 1 { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Cannot create threadpool with no threads")) };
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Ok(ThreadPool {_workers: workers, sender: Some(sender)})
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self._workers {
            debug!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker._thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    _thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            info!("Waiting");
            match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    trace!("Worker {id} got job");
                    job();
                },
                Err(_) => {
                    trace!("Channel closed, worker {id} quitting");
                    break
                },
            }
        });
        Worker {id, _thread: Some(thread)}
    }
}