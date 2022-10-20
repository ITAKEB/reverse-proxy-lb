use std::net::TcpListener;
use std::collections::VecDeque;

use reverse_proxy_lb::config::{ IP_LISTENER, IP_SERVERS, NUM_THREADS };
use reverse_proxy_lb::threadpool::{ ThreadPool, read_ip_server };
use reverse_proxy_lb::connecting::handle_connection;

fn main() {
    match TcpListener::bind(IP_LISTENER) {
        Ok(listener) => {
            let pool = ThreadPool::new(NUM_THREADS);
            let (push, pop) = read_ip_server();
            handle_connection(pool, listener, &push, &pop);
        },

        Err(_) => { println!("Failed to listen in {}", IP_LISTENER) },
    }
}
