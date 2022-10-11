use std::collections::VecDeque;
use std::io::{Error, ErrorKind};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::config::IP_SERVERS;

type Job = Box<dyn FnOnce(&String) + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, std::io::Error> {
        if size > 0 {
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));
            let mut workers = Vec::with_capacity(size);

            let mut ip_servers: VecDeque<String> = VecDeque::with_capacity(3);

            for ip in IP_SERVERS {
                ip_servers.push_back(ip.to_string());
            }

            let mut mux_ip_servers = Arc::new(Mutex::new(ip_servers));

            for id in 0..size {
                workers.push(Worker::new(
                    id,
                    Arc::clone(&receiver),
                    Arc::clone(&mut mux_ip_servers),
                ));
            }

            Ok(ThreadPool {
                workers,
                sender: Some(sender),
            })
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "It's not possible create threadpool",
            ))
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&String) + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        ip_servers: Arc<Mutex<VecDeque<String>>>,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    let ip: String = ip_servers
                        .lock()
                        .unwrap()
                        .pop_front()
                        .expect("empty value returned");
                    job(&ip);
                    ip_servers.lock().unwrap().push_back(ip);
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

