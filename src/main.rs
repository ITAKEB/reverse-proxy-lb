use anyhow::*;
use std::borrow::BorrowMut;
use std::result::Result::Ok;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut stream, _addr) = listener.accept().await?;
        //println!("new client: {:?}", stream);
        println!("{:?}", stream.local_addr()?);
        let strm = stream.borrow_mut();
        let mut reader = BufReader::new(strm);

        let mut request = String::new();
        reader.read_line(&mut request).await?;
        //Replace Host
        let mut host = String::new();
        reader.read_line(&mut host).await?;
        host = "Host: 172.253.115.91\r\n".to_owned();

        request.push_str(&host);
        loop {
            reader.read_line(&mut request).await?;
            if request.ends_with("\r\n\r\n") {
                break;
            }
        }
        println!("-------------------------");
        println!("request: {:?}", request);

        //Google IP Adress on port 80 (http comunication)
        let mut server = TcpStream::connect("172.253.115.91:80").await?;

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
    }

    Ok(())
}
