use std::net::TcpStream;
use std::collections::HashMap;
use std::io::{ BufReader, BufRead, Write };

pub fn read_request(stream: &TcpStream, ip_server: &str) -> Result<(String, String), std::io::Error> {
    let mut buf_reader = BufReader::new(stream);
    let mut req_client = String::new();
    let mut req_server = String::new();
    let mut headers = String::new();

    buf_reader.read_line(&mut req_client);
    req_server = req_client.clone();

    //let ip_server = "127.0.0.1:8000";
    let host = format!("Host: {}\r\n", ip_server);
    req_server.push_str(&host);

    loop {
        buf_reader.read_line(&mut req_client)?;
        if req_client.ends_with("\r\n\r\n") {
            break;
        }
    }

    req_server = req_client.clone();

    Ok((req_client, req_server))
}

pub fn parsing_req(mut stream: TcpStream, mut st: &TcpStream) -> (Vec<u8>, String, BufReader<TcpStream>) {
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();

    loop {
        reader.read_line(&mut response).unwrap();
        if response.ends_with("\r\n\r\n") {
            break;
        }
    }

    //Save headers
    let mut headers: HashMap<String, String> = HashMap::new();
    let lines: Vec<String> = response
        .split("\r\n")
        .map(|s| s.to_owned())
        .collect();

    for i in 0..lines.len() - 1 {
        let (k, v) = lines[i]
            .split_once(':')
            .unwrap_or(("response_code", &lines[0]));

        headers.insert(k.to_lowercase(), v.trim().to_string());
    }

    //Search content-len body
    let header = headers.get("content-length");

    let content_len = match header {
        Some(s) => s,
        None => "0",
    };

    let mut body = vec![0; content_len.parse().unwrap_or(0)];

    (body, response, reader)
    //match reader.read_exact(&mut body) {
    //    Ok(_) => {
    //        let response_bytes = [response.as_bytes(), &body].concat();
    //        let responsing = st.write_all(&response_bytes);
    //    },
    //    Err(_) => {
    //        println!("error");
    //    },
    //}
}
