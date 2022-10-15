use crate::threadpool::ThreadPool;
use std::net::{ TcpListener, TcpStream };
use std::io::{ BufReader, Write };

use crate::request::{ read_request, parsing_request };
use crate::http_error::error_503;

pub fn http_connect(ip_server: String, stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request = read_request(buf_reader, &ip_server);

    match request {
        Ok(req) => {
            let connection = TcpStream::connect(&ip_server);
            match connection {
                Ok(mut cn) => {
                    match cn.write_all(req.as_bytes()) {
                        Ok(_) => {
                            let mut reader = BufReader::new(&cn);
                            match parsing_request(cn, stream) {
                                Ok(_) => {
                                    println!("send")
                                },
                                Err(_) => {
                                    //error_503(stream);
                                }
                            }
                        },
                        Err(_) => {
                            error_503(stream);
                        }
                    }
                },
                Err(_) => {
                    error_503(stream);
                }
            }
        },
        Err(_) => {
            error_503(stream);
        },
    }
}

pub fn handle_connection(pool: ThreadPool, listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(st) => {
                pool.execute(|ip| {
                    http_connect(ip, st);
                });
            }
            Err(_) => continue,
        }
    }
}
