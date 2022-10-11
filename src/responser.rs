use std::fs;
use std::io::{prelude::*, BufReader};
use std::net::TcpStream;

use crate::config::PORT;

fn write_log_file(request: &String, type_data: String) -> Result<(), std::io::Error> {
    let mut old_text = fs::read_to_string("./files/log.txt")?;

    old_text.push_str(&type_data);
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

fn try_write_log_file(info: &String, type_data: String) {
    match write_log_file(&info, type_data) {
        Ok(_) => println!("Saving in file log"),
        Err(_) => println!("It's not possible save in file log"),
    }
}

fn try_send_response(mut stream: &TcpStream, response: String, type_data: String) {
    match stream.write_all(response.as_bytes()) {
        Ok(_) => { 
            try_write_log_file(&response, type_data);
            println!("response proxy: {}", response); 
        },
        Err(_) => { println!("Response does not send") },
    }
}

fn try_send_response_web(mut stream: &TcpStream, response: Vec<u8>) {
    match stream.write_all(&response) {
        Ok(_) => { 
            //try_write_log_file(&response, type_data);
            println!("response Server: {:?}", response); 
        },
        Err(_) => { println!("Response does not send") },
    }
}

pub fn handle_connection(mut stream: TcpStream, ip_server: &String) {
    let buf_reader = BufReader::new(&stream);
    let request = read_request(buf_reader, ip_server);

    match request {
        Ok(req) =>{
            try_write_log_file(&req, String::from("Request proxy\r\n"));

            let addr = format!("{}:{}", ip_server, PORT);
            let connection = TcpStream::connect(&addr);

            match connection {
                Ok(mut cn) => {
                    let mut response = Vec::new();
                    match cn.write_all(req.as_bytes()) {
                        Ok(_) => {
                            match cn.read_to_end(&mut response) {
                                Ok(_) => {
                                    try_send_response_web(&stream, response);
                                    //stream.write_all(&response).unwrap();
                                },
                                Err(_) => { 
                                    let response = String::from("HTTP/1.1 503 Service Unavailable\r\n\r\n");
                                    try_send_response(&stream, response, String::from("Response proxy\r\n"));
                                },
                            }
                        },
                        Err(_) => {
                            let response = String::from("HTTP/1.1 503 Service Unavailable\r\n\r\n");
                            try_send_response(&stream, response, String::from("Response proxy\r\n"));
                        },
                    }
               },
                Err(_) => {
                    let response = String::from("HTTP/1.1 503 Service Unavailable\r\n\r\n");
                    try_send_response(&stream, response, String::from("Response proxy\r\n"));
                }
            }
       },
        Err(_) => {
            let response = String::from("HTTP/1.1 400 Bad Request\r\n\r\n");
            try_send_response(&stream, response, String::from("Response proxy\r\n"));
        },
    }

    match stream.flush() {
        Ok(_) => println!("Flush done"),
        Err(_) => println!("Flush error"),
    };
}

