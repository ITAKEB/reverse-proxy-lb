use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};

use reverse_proxy_lb::config::{IP_LISTENER, PORT};
use reverse_proxy_lb::threadpool::ThreadPool;

fn write_log_file(request: &String) -> Result<(), std::io::Error> {
    let mut old_text = fs::read_to_string("./files/log.txt")?;

    old_text.push_str(&request);
    fs::write("./files/log.txt", old_text)?;

    Ok(())
}

fn read_request(
    mut buf_reader: BufReader<&TcpStream>,
    ip_server: &String,
) -> Result<String, std::io::Error> {
    let mut request = String::new();
    buf_reader.read_line(&mut request)?;
    let mut host = String::new();
    buf_reader.read_line(&mut host)?;

    host = format!("Host: {}\r\n", ip_server);
    request.push_str(&host);
    loop {
        buf_reader.read_line(&mut request)?;
        if request.ends_with("\r\n\r\n") {
            break;
        }
    }

    Ok(request)
}

fn handle_connection(mut stream: TcpStream, ip_server: &String) {
    let buf_reader = BufReader::new(&stream);
    let request = read_request(buf_reader, ip_server);

    match request {
        Ok(rq) => match write_log_file(&rq) {
            _ => {
                let connection = TcpStream::connect(&ip_server);
                match connection {
                    Ok(mut cn) => {
                        let mut response = Vec::new();
                        cn.read_to_end(&mut response).unwrap();
                        stream.write_all(&response).unwrap();
                    }
                    Err(_) => {
                        let response = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                }
            }
        },
        Err(_) => {
            let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }

    stream.flush().unwrap();
}

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
