use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use std::collections::VecDeque;

use crate::config::IP_SERVERS;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce(String) + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        let mut ip_servers: VecDeque<String> = VecDeque::with_capacity(3);

        for ip in IP_SERVERS {
            ip_servers.push_back(ip.to_string());
        }

        let mux_ip_servers = Arc::new(Mutex::new(ip_servers));

        for id in 0..size {
            workers.push(
                Worker::new(
                    id, 
                    Arc::clone(&receiver),
                    Arc::clone(&mux_ip_servers)
                    ));
        }

        ThreadPool { 
            workers, 
            sender 
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(String) + Send + 'static,
    {
        let job = Box::new(f);
        if let Err(_) = self.sender.send(job) {
            println!("Error: Did not send job");
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, ip_servers: Arc<Mutex<VecDeque<String>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            if let Ok(lock) = receiver.lock() {
                if let Ok(job) = lock.recv() {
                    drop(lock);
                    println!("Executing job in Worker: {id}");

                    if let Ok(mut queue) = ip_servers.lock() {
                        if let Some(ip) = queue.pop_front() {
                            drop(queue);
                            ip_servers.lock().unwrap().push_back(ip.clone());
                            job(ip);
                        } else {
                            println!("Ip's for web server never was received");
                        }
                    } else {
                        println!("Ip for web server did not find");
                    }
                } else {
                    println!("Job never was received");
                }
            } else {
                println!("Iw was not possible lock thread");
            }
        });

        Worker { 
            id, 
            thread 
        }
    }
}
