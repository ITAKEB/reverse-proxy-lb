use std::net::{ TcpListener, TcpStream }; 
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{ SyncSender, Receiver };
use std::io::Write;

use crate::threadpool::ThreadPool;
use crate::request::{ read_request, parsing_req };

pub fn http_connect(mut stream: TcpStream, 
    push: SyncSender<&'static str>, 
    pop: Arc<Mutex<Receiver<&'static str>>>
) {
    if let Ok(lock) = pop.lock() {
        if let Ok(ip_server) = lock.recv() {
            drop(lock);
            push.send(ip_server.clone()).unwrap();
            match read_request(&stream, ip_server) {
                Ok((req_client, req_server)) => {
                    let connection = TcpStream::connect(&ip_server);
                    match connection {
                        Ok(mut cn) => {
                            match cn.write_all(req_server.as_bytes()) {
                                Ok(_) =>{
                                    println!("send req");
                                    let (mut body, header, mut reader) = parsing_req(cn, &stream); 
                                    println!("send response");
                                    match reader.read_exact(&mut body) {
                                        Ok(_) => {
                                            let response_bytes = [header.as_bytes(), &body].concat();
                                            stream.write_all(&response_bytes);
                                            println!("done");
                                        },
                                        Err(_) => {
                                            println!("error");
                                        },
                                    }
                                },
                                Err(_) => {},
                            }
                        },
                        Err(_) => { println!("Do not connected to web server"); },
                    }
                },
                Err(_) => { println!("Not possible read client request") },
            }
        }
    }

    stream.flush().unwrap();
}

pub fn handle_connection(
    pool: ThreadPool, 
    listener: TcpListener, 
    push: &SyncSender<&'static str>, 
    pop: &Arc<Mutex<Receiver<&'static str>>>
) {
    for stream in listener.incoming() {
        match stream {
            Ok(st) => {
                let pop_clone = Arc::clone(pop);
                let push_clone = push.clone();

                pool.execute(|| {
                    http_connect(st, push_clone, pop_clone);
                });
            }
            Err(_) => { println!("Stream does not capture") },
        }
    }
}
