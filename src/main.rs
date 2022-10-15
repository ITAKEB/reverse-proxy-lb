use std::net::TcpListener;

use reverse_proxy_lb::config::{ IP_LISTENER, NUM_THREADS};
use reverse_proxy_lb::responser::handle_connection;
use reverse_proxy_lb::threadpool::ThreadPool;

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind(IP_LISTENER)?;
    let pool = ThreadPool::new(NUM_THREADS);

    handle_connection(pool, listener);

    Ok(())
}
