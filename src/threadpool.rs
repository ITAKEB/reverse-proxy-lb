use std::sync::mpsc::{Receiver, SyncSender};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::config::IP_SERVERS;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Err(_) = self.sender.send(job) {
            println!("Error: Did not send job");
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            if let Ok(lock) = receiver.lock() {
                if let Ok(job) = lock.recv() {
                    drop(lock);
                    job();
                } else {
                    println!("Job never was received");
                }
            } else {
                println!("It was not possible lock thread");
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub fn read_ip_server() -> (SyncSender<&'static str>, Arc<Mutex<Receiver<&'static str>>>) {
    let (push, receiver) = mpsc::sync_channel(IP_SERVERS.len());

    for ip in IP_SERVERS {
        push.send(ip).unwrap();
    }

    let pop = Arc::new(Mutex::new(receiver));

    (push, pop)
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
