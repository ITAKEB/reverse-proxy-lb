use std::io::{prelude::*, BufReader};
use std::net::TcpStream;
use std::collections::HashMap;

pub fn read_request(
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

pub fn parsing_request(mut stream: TcpStream, mut st: TcpStream) -> Result<(), std::io::Error> {
    let mut reader = BufReader::new(stream);
    let mut response = String::new();

    reader.read_line(&mut response)?;
    loop {
        reader.read_line(&mut response)?;
        if response.ends_with("\r\n\r\n") {
            break;
        }
    }

    //Save headers
    let mut headers: HashMap<String, String> = HashMap::new();
    let lines: Vec<String> = response.split("\r\n").map(|s| s.to_owned()).collect();
    for i in 0..lines.len() - 1 {
        let (k, v) = lines[i]
            .split_once(':')
            .unwrap_or(("response_code", &lines[0]));

        headers.insert(k.to_lowercase(), v.trim().to_string());
    }

    println!("-------------------------");
    println!("headers = {:?}", headers);
    println!("-------------------------");

    //Search content-len body
    let header = headers.get("content-length");

    let content_len = match header {
        Some(s) => s,
        None => "0",
    };

    println!("response len = {:?}", response.as_bytes().len());
    println!("content len = {:?}", content_len);
    //Create byte array to save content_body with especific size
    let mut body = vec![0; content_len.parse().unwrap_or(0)];

    match reader.read_exact(&mut body) {
        Ok(_) => {
            println!("-------------------------");
            println!("buffer_body len = {:?}", body.len());
            println!("-------------------------");

            let response_bytes = [response.as_bytes(), &body].concat();
            let responsing = st.write_all(&response_bytes);
        },
        Err(_) => {
            println!("error");
        },
    }
    Ok(())
}
