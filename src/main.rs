use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time;

use reverse_proxy_lb::threadpool::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "./files/hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "./files/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    //let ten_millis = time::Duration::from_millis(10000);
    //thread::sleep(ten_millis);
 
    stream.flush().unwrap();
}

fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(3)?;

    for stream in listener.incoming() {
        match stream {
            Ok(st) => {
                pool.execute(|| {                    
                    handle_connection(st);

                });
            },
            Err(_) => continue,
        }
    }
    Ok(())
}
