use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::fs;

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

fn parse_response(request: &String) -> HashMap<String, String> {
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
    let mut writer = BufWriter::new(st_server);
    let size = req_bytes.len();
    let buff_size = if size < 2048 { size } else { size / 1024 };

    for chunk in req_bytes.chunks(buff_size) {
        let mut pos = 0;
        while pos < chunk.len() {
            if let Ok(bytes_written) = writer.write(&chunk[pos..]) {
                pos += bytes_written;
                if let Err(_) = writer.flush() {
                    println!("Failed to flush request buffer responser");
                    return;
                }
            } else {
                println!("Failed to write request");
                return;
            }
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
    if let Err(_) = st_client.write(status_line.as_bytes()) {
        println!("Failed to send error response");
    }
}


fn write_resp_log(req: &String, req_head: &String,  type_req: String) {
    let req_total = format!("\r\n{}\r\n{}{}", type_req, req, req_head);
    if let Ok(mut old_text) = fs::read_to_string("./files/log.txt") {
        old_text.push_str(&req_total);
        if let Err(_) = fs::write("./files/log.txt", old_text) {
            println!("Failed write log");
        }
    }
}
