use std::net::TcpListener;

use reverse_proxy_lb::config::{IP_LISTENER, PORT};
use reverse_proxy_lb::threadpool::ThreadPool;
use reverse_proxy_lb::responser::handle_connection;

fn main() -> Result<(), std::io::Error> {
    let addr_listener = format!("{}:{}", IP_LISTENER, PORT);
    let listener = TcpListener::bind(addr_listener)?;
    let pool = ThreadPool::new(3)?;

    for stream in listener.incoming() {
        match stream {
            Ok(st) => {
                pool.execute(|ip| {
                    handle_connection(st, ip);
                });
            }
            Err(_) => continue,
        }
    }
    Ok(())
}
