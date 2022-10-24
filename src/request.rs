use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

pub fn read_request(
    mut st_client: &TcpStream,
) -> Result<(String, HashMap<String, String>, Vec<u8>), std::io::Error> {
    let mut buf_reader = BufReader::new(&mut st_client);
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

    write_req_log(&req_head, &req, "Request Client".to_string());
    let headers = parse_request(&req);
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

fn parse_request(request: &String) -> HashMap<String, String> {
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

pub fn write_request(
    req_head: &mut String,
    headers: &mut HashMap<String, String>,
    st_server: &TcpStream,
    ip: String,
    body: Vec<u8>,
) {
    let mut headers_str = String::new();
    for (key, value) in headers.clone() {
        headers_str.push_str(&format!("{}{}\r\n", key, value));
    }

    write_req_log(&req_head, &headers_str, "Request Proxy".to_string());
    remove_header(headers);
    headers.insert("host".to_string(), ip);
    let req_bytes = concat_req(req_head, headers, body);
    let mut buf_writer = BufWriter::new(st_server);
    let size = req_bytes.len();
    let buff_size = if size < 2048 { size } else { size / 1024 };

    for chk in req_bytes.chunks(buff_size) {
        let mut total_bytes_written = 0;
        while total_bytes_written < chk.len() {
            if let Ok(bytes_written) = buf_writer.write(&chk[total_bytes_written..]) {
                total_bytes_written += bytes_written;
                if let Err(_) = buf_writer.flush() {
                    println!("Failed to flush BufWriter");
                    return;
                }
            } else {
                println!("Failed to write request for Web Server");
                return;
            }
        }
    }
}

fn remove_header(req: &mut HashMap<String, String>) {
    req.remove(&"transfer-encoding".to_string());
    req.remove(&"accept-encoding".to_string());
    req.remove(&"content-encoding".to_string());
    req.remove(&"upgrade".to_string());
}

fn hashmap_to_vec(bytes: &mut Vec<u8>, headers: &mut HashMap<String, String>) {
    for (k, v) in headers {
        let mut line = format!("{k}:{v}\r\n").as_bytes().to_vec();
        bytes.append(&mut line);
    }

    bytes.push(0x0D);
    bytes.push(0x0A);
}

fn concat_req(
    req_head: &mut String,
    headers: &mut HashMap<String, String>,
    mut body: Vec<u8>,
) -> Vec<u8> {
    let mut req_bytes = req_head.as_bytes().to_vec();
    hashmap_to_vec(&mut req_bytes, headers);
    req_bytes.append(&mut body);
    req_bytes
}

pub fn write_req_log(req: &String, req_head: &String, type_req: String) {
    let req_total = format!("{}\r\n{}{}", type_req, req, req_head);
    if let Ok(mut old_text) = fs::read_to_string("./files/log.txt") {
        old_text.push_str(&req_total);
        if let Err(_) = fs::write("./files/log.txt", old_text) {
            println!("Failed write log");
        }
    }
}
