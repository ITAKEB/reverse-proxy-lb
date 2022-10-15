use std::net::TcpStream;
use std::io::Write;

pub fn error_503(mut stream: TcpStream) {
    let response = "HTTP/1.1 503 Service Unavailable\r\n\r\n";
    match stream.write_all(response.as_bytes()) {
        Ok(_) => { 
            //try_write_log_file(&response, type_data);
        },
        Err(_) => { println!("Response does not send") },
    }
}
