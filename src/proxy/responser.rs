use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use crate::cache::filedata::FileData;

use crate::proxy::config::DIR_LOG;

pub fn read_response(
    mut stream: &TcpStream,
) -> Result<(String, HashMap<String, String>, Vec<u8>), std::io::Error> {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut req: String = String::new();
    let mut req_head: String = String::new();
    buf_reader.read_line(&mut req_head)?;
    buf_reader.read_line(&mut req_head)?;
    buf_reader.read_line(&mut req_head)?;

    loop {
        buf_reader.read_line(&mut req)?;
        if req.ends_with("\r\n\r\n") {
            break;
        }
    }

    write_resp_log(&req_head, &req, "Response Web Server".to_string());
    let headers = parse_response(&req);
    let content_length = get_content_length(&headers);
    let mut body = vec![0; content_length];

    buf_reader.read_exact(&mut body)?;

    Ok((req_head, headers, body))
}

fn get_content_length(req: &HashMap<String, String>) -> usize {
    match req.get("content-length") {
        Some(s) => s.parse().unwrap_or(0),
        None => 0,
    }
}

fn parse_response(request: &str) -> HashMap<String, String> {
    let mut headers: HashMap<String, String> = HashMap::new();
    let lines: Vec<String> = request.split("\r\n").map(|s| s.to_owned()).collect();

    for ln in 0..lines.len() - 1 {
        let (k, v) = lines[ln]
            .split_once(':')
            .unwrap_or(("response_code", &lines[0]));

        headers.insert(k.to_lowercase(), v.trim().to_string());
    }

    headers
}

pub fn write_response(
    req_head: &mut String,
    headers: &mut HashMap<String, String>,
    st_server: &TcpStream,
    body: Vec<u8>,
) {
    let req_bytes = concat_resp(req_head, headers, body);
    let mut buf_writer = BufWriter::new(st_server);
    let size = req_bytes.len();
    let buff_size = if size < 2048 { size } else { size / 1024 };

    for chk in req_bytes.chunks(buff_size) {
        let mut total_bytes_written = 0;
        while total_bytes_written < chk.len() {
            if let Ok(bytes_written) = buf_writer.write(&chk[total_bytes_written..]) {
                total_bytes_written += bytes_written;
                if buf_writer.flush().is_err() {
                    println!("Failed to flush BufWriter");
                    return;
                }
            } else {
                println!("Failed to write response");
                return;
            }
        }
    }
}

pub fn write_response_from_file(stream: &TcpStream, filedata: FileData, map: &mut HashMap<String, String>) {

    let version = "HTTP/1.1".to_string();
    let code = "200";
    let response = "OK".to_string();

    map.insert("server".to_string(), "reverse-proxy-lb".to_string());

    if let Some(content_type) = filedata.metadata.content_type {

        map.insert("content-type".to_string(), content_type);
    }

    map.insert("content-length".to_string(), filedata.metadata.content_length.to_string());

    let mut buf_writer = BufWriter::new(stream);

    let mut status = {

        let version = version.as_bytes();
        let code = code.as_bytes();
        let response = response.as_bytes();
        let sp = [b' '];
        let crlf = [0x0D, 0x0A];
        let line = [version, &sp, code, &sp, response, &crlf].concat();

        line.to_vec()
    };

    let content = {

        let mut temp = {
            for (key, value) in map.iter() {

                let mut element = format!("{key}:{value}\r\n").as_bytes().to_vec();
                status.append(&mut element);
            }

            status.push(0x0D);
            status.push(0x0A);

            status
        };

        let mut body = filedata.content_data.clone();
        temp.append(&mut body);

        temp
    };

    let size = content.len();
    let bf_size = if size < 2048 { size } else { size / 1024 };

    for chunk in content.chunks(bf_size) {

        let mut total_bytes_written = 0;

        while total_bytes_written < chunk.len() {

            if let Ok(bytes_written) = buf_writer.write(&chunk[total_bytes_written..]) {

                total_bytes_written += bytes_written;

                if buf_writer.flush().is_err() { println!("Failed to flush BufWriter"); return; }
            } else { println!("Failed to write response"); return; }
        }
    }
}

fn hashmap_to_vec(bytes: &mut Vec<u8>, headers: &mut HashMap<String, String>) {
    for (k, v) in headers {
        let mut line = format!("{k}:{v}\r\n").as_bytes().to_vec();
        bytes.append(&mut line);
    }

    bytes.push(0x0D);
    bytes.push(0x0A);
}

fn concat_resp(
    req_head: &mut String,
    headers: &mut HashMap<String, String>,
    mut body: Vec<u8>,
) -> Vec<u8> {
    let mut req_bytes = req_head.as_bytes().to_vec();
    hashmap_to_vec(&mut req_bytes, headers);
    req_bytes.append(&mut body);
    req_bytes
}

pub fn write_error(status_line: String, st_client: &mut TcpStream) {
    if st_client.write(status_line.as_bytes()).is_err() {
        println!("Failed to send error response");
    }
}

fn write_resp_log(req: &String, req_head: &String, type_req: String) {
    let req_total = format!("\r\n{}\r\n{}{}", type_req, req, req_head);
    if let Ok(mut old_text) = fs::read_to_string(DIR_LOG) {
        old_text.push_str(&req_total);
        if fs::write(DIR_LOG, old_text).is_err() {
            println!("Failed write log");
        }
    } else {
        println!("Failed to find {}", DIR_LOG);
    }
}

pub fn write_resp_err_log(error: &String, ip: &str) {
    let text = format!("Response Proxy: {}\r\n{}", ip, error);
    if let Ok(mut old_text) = fs::read_to_string(DIR_LOG) {
        old_text.push_str(&text);
        if fs::write(DIR_LOG, old_text).is_err() {
            println!("Failed write log");
        }
    } else {
        println!("Failed to find {}", DIR_LOG);
    }

}

pub fn write_failed_to_connect(st_client: &mut TcpStream) {
    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("failed.html").unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    if st_client.write(response.as_bytes()).is_err() {
        println!("Failed to send error response");
    }
}
