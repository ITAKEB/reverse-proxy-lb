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
    }

    Ok(())
}
