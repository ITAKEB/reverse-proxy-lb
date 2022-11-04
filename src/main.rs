use std::net::TcpListener;
use std::path::Path;
use std::sync::mpsc;

use reverse_proxy_lb::proxy::config::{IP_LISTENER, NUM_THREADS};
use reverse_proxy_lb::proxy::connecting::handle_connection;
use reverse_proxy_lb::proxy::threadpool::{read_ip_server, ThreadPool};
use reverse_proxy_lb::cache::utils::run_writer;
use reverse_proxy_lb::cache::utils::run_cleaner;

fn main() {
    match TcpListener::bind(IP_LISTENER) {
        Ok(listener) => {
            println!("Listening in {}", IP_LISTENER);
            let pool = ThreadPool::new(NUM_THREADS);
            let (push, pop) = read_ip_server();
            let (sender, receiver) = mpsc::channel();

            let path = String::from(r"./cachefiles");
            let ttl = 180 as u64;
            let cache_dir = Path::new(path.as_str());
            run_cleaner(<&std::path::Path>::clone(&cache_dir).to_path_buf());
            run_writer(receiver);

            handle_connection(pool, listener, &push, &pop, &sender, cache_dir.to_path_buf(), ttl, true);
        }
        Err(_) => println!("Failed to listen in {}", IP_LISTENER),
    }
}
