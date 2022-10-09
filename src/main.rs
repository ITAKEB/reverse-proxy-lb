use anyhow::*;

use std::result::Result::Ok;
use std::time::Duration;
use std::{borrow::BorrowMut, collections::VecDeque};
use std::{fs, thread};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut rrb: VecDeque<String> = VecDeque::with_capacity(3);

    let servers_ip = read_config_file()?;
    //First server
    rrb.push_back(servers_ip[0].clone());
    //Second server
    rrb.push_back(servers_ip[1].clone());
    //Third server
    rrb.push_back(servers_ip[2].clone());

    //println!("{:?}", rrb);

    loop {
        let (mut stream, _addr) = listener.accept().await?;
        //println!("new client: {:?}", stream);
        //println!("{:?}", stream.local_addr()?);
        let strm = stream.borrow_mut();

        let mut reader = BufReader::new(strm);

        let mut request = String::new();
        reader.read_line(&mut request).await?;
        //Replace Host
        let mut host = String::new();
        reader.read_line(&mut host).await?;
        //host = "Host: 54.174.150.245\r\n".to_owned();
        //Pop new server ip
        let ip_server: String = rrb.pop_front().unwrap();
        let ip: Vec<&str> = ip_server.split(':').collect();

        host = format!("Host: {}\r\n", ip[0]);
        request.push_str(&host);
        loop {
            reader.read_line(&mut request).await?;
            if request.ends_with("\r\n\r\n") {
                break;
            }
        }
        println!("-------------------------");
        println!("request: {:?}", request);
        //Save request on log.txt
        write_log_file(&request)?;

        //Server ip conection (http comunication)
        let conection = TcpStream::connect(&ip_server).await;

        match conection {
            Ok(mut server) => {
                server.write_all(request.as_bytes()).await?;
                let mut reader = BufReader::new(server);
                let mut response = String::new();
                reader.read_line(&mut response).await?;
                loop {
                    reader.read_line(&mut response).await?;
                    if response.ends_with("\r\n\r\n") {
                        break;
                    }
                }
                println!("-------------------------");
                println!("response: {:?}", response);

                let mut cursor = Cursor::new(reader.buffer());
                let mut body = vec![];
                loop {
                    let num_bytes = cursor
                        .read_until(b'-', &mut body)
                        .await
                        .expect("reading from cursor won't fail");

                    if num_bytes == 0 {
                        break;
                    }
                }
                println!("-------------------------");
                let b = String::from_utf8(body)?;
                println!("body = {:?}", b);

                response.push_str(&b);
                stream.write_all(response.as_bytes()).await?;
                rrb.push_back(ip_server);
                println!("-------------------------");
                println!("{:?}", rrb);
            }
            Err(err) => {
                println!("Internal error: {}", err);
                let response = String::from("HTTP/1.1 500 Internal Server Error\r\n\r\n");

                stream.write_all(response.as_bytes()).await?;
                //Add to end ip server
                rrb.push_back(ip_server);
                println!("-------------------------");
                println!("{:?}", rrb);
            }
        }
        thread::sleep(Duration::from_millis(2000));
    }

    Ok(())
}

fn read_config_file() -> Result<Vec<String>> {
    let data = fs::read_to_string("./files/config.txt").expect("Unable to read file");

    let ips = data.split("\n").map(|s| s.to_owned()).collect();

    Ok(ips)
}

fn write_log_file(data: &String) -> Result<()> {
    let mut old_text = fs::read_to_string("./files/log.txt").expect("Unable to read file");

    old_text.push_str(data);
    fs::write("./files/log.txt", old_text).expect("Unable to write file");

    Ok(())
}
