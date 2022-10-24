use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    mpsc::{Receiver, SyncSender},
    Arc, Mutex,
};
use std::time;

use crate::request::{read_request, write_request};
use crate::responser::{read_response, write_response, write_error};
use crate::threadpool::ThreadPool;

fn connect_to_server(ip: &str, retries: u16) -> Result<TcpStream, std::io::Error> {
    if let Ok(st_server) = TcpStream::connect(&ip) {
        return Ok(st_server);
    } else {
        if retries < 1 {
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to establish connection with web server",
            ));
        } else {
            let dur = time::Duration::from_millis(2000);
            std::thread::sleep(dur);
            connect_to_server(ip, retries - 1)
        }
    }
}

pub fn http_connect(
    st_client: &mut TcpStream,
    push: SyncSender<&'static str>,
    pop: Arc<Mutex<Receiver<&'static str>>>,
) {
    if let Ok(lock) = pop.lock() {
        if let Ok(ip_server) = lock.recv() {
            drop(lock);
            push.send(ip_server.clone()).unwrap();
            match read_request(&st_client) {
                Ok((mut req_head, mut header, body)) => {
                    let st_server = connect_to_server(&ip_server, 3);
                    match st_server {
                        Ok(server) => {
                            write_request(
                                &mut req_head,
                                &mut header,
                                &server,
                                ip_server.to_string(),
                                body,
                            );
                            match read_response(&server) {
                                Ok((mut req_head, mut header, body)) => {
                                    write_response(&mut req_head, &mut header, &st_client, body);
                                }
                                Err(_) => {write_error("HTTP/1.1 503 502 Bad Gateway".to_string(), st_client)},
                            }
                        }
                        Err(_) => {write_error("HTTP/1.1 503 Service Unavailable".to_string(), st_client)}
                    }
                }
                Err(_) => {write_error("HTTP/1.1 400 Bad Request".to_string(), st_client)},
            }
        }
    }
}

pub fn handle_connection(
    pool: ThreadPool,
    listener: TcpListener,
    push: &SyncSender<&'static str>,
    pop: &Arc<Mutex<Receiver<&'static str>>>,
) {
    for stream in listener.incoming() {
        match stream {
            Ok(mut st) => {
                let pop_clone = Arc::clone(pop);
                let push_clone = push.clone();

                pool.execute(move || {
                    http_connect(&mut st, push_clone, pop_clone);
                });
            }
            Err(_) => println!("Stream does not capture"),
        }
    }
}
