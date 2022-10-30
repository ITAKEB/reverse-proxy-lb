use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{
    mpsc::{Receiver, SyncSender},
    Arc, Mutex,
};
use std::time;
use std::path::{PathBuf, Path};

use crate::proxy::request::{read_request, write_request, is_cache_request, parse_cache_info};
use crate::proxy::responser::{read_response, write_error, write_response};
use crate::proxy::threadpool::ThreadPool;
use crate::cache::metadata::Metadata;
use super::responser::write_response_from_file;
use crate::cache::filedata::{create_file_path, FileData};

fn connect_to_server(ip: &str, retries: u16) -> Result<TcpStream, std::io::Error> {
    if let Ok(st_server) = TcpStream::connect(&ip) {
        Ok(st_server)
    } else if retries < 1 {
            Err(Error::new(ErrorKind::Other, "Failed to establish connection with web server"))
        } else {
            let dur = time::Duration::from_millis(2000);
            std::thread::sleep(dur);
            connect_to_server(ip, retries - 1)
        }
}

pub fn http_connect(
    st_client: &mut TcpStream,
    push: SyncSender<&'static str>,
    pop: Arc<Mutex<Receiver<&'static str>>>,
    cache_sender: Sender<FileData>,
    cache_folder: PathBuf,
    ttl: u64,
    ) {
    if let Ok(lock) = pop.lock() {
        if let Ok(ip_server) = lock.recv() {
            drop(lock);
            push.send(<&str>::clone(&ip_server)).unwrap();

            if let Ok((mut req_head, mut header, body)) = read_request(st_client) {

                let mut map:HashMap<String, String> = HashMap::new();
                let info = parse_cache_info(&req_head);

                let file_path = create_file_path(&cache_folder, info.1.clone());
                let route = file_path.clone();

                if let Ok(metadata) = Metadata::parse_file(&file_path) {
                    if !metadata.ttl_check() {
                        if is_cache_request(&info.0) {
                            if let Ok(filedata) = FileData::parse_file(file_path, metadata) {
                                write_response_from_file(st_client, filedata, &mut map);
                            } else { handle_file(st_client, ip_server, &mut req_head, &mut header, body, cache_sender, &route, &map, ttl); }
                        } else { handle_file(st_client, ip_server, &mut req_head, &mut header, body, cache_sender, &route, &map, ttl); }
                    } else { handle_file(st_client, ip_server, &mut req_head, &mut header, body, cache_sender, &route, &map, ttl); }
                } else { handle_file(st_client, ip_server, &mut req_head, &mut header, body, cache_sender, &route, &map, ttl); }
            }
        }
    }
}

pub fn handle_connection(
    pool: ThreadPool,
    listener: TcpListener,
    push: &SyncSender<&'static str>,
    pop: &Arc<Mutex<Receiver<&'static str>>>,
    cache_sender: &Sender<FileData>,
    cache_folder: PathBuf,
    ttl: u64,
) {
    for stream in listener.incoming() {
        match stream {
            Ok(mut st) => {
                let pop_clone = Arc::clone(pop);
                let push_clone = push.clone();
                let sender = cache_sender.clone();
                let cache = cache_folder.clone();
                pool.execute(move || {
                    http_connect(
                        &mut st,
                        push_clone,
                        pop_clone,
                        sender,
                        cache,
                        ttl,
                    );
                });
            }
            Err(_) => println!("Stream does not capture"),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_file(
    st_client: &mut TcpStream,
    ip_server: &str,
    req_head: &mut String,
    header: &mut HashMap<String, String>,
    body: Vec<u8>,
    cache_sender: Sender<FileData>,
    path: &Path,
    map: &HashMap<String, String>,
    ttl: u64,
) {

    match connect_to_server(ip_server, 3) {
        Ok(server) => {
            write_request(
                req_head,
                header,
                &server,
                ip_server.to_string(),
                body,
            );
            match read_response(&server) {
                Ok((mut req_head, mut header, body)) => {

                    let len = body.len();
                    let temp_body = body.clone();

                    if let Ok(filedata) = FileData::default(ttl, len as u64, path.to_path_buf(), temp_body, map.get(&"content-type".to_string()).cloned()) {

                        if cache_sender.send(filedata).is_err() {
                            println!("Failed to queue cache file");
                        }
                    }

                    write_response(&mut req_head, &mut header, st_client, body);
                }
                Err(_) => {
                    write_error("HTTP/1.1 502 Bad Gateway".to_string(), st_client)
                }
            }
        }
        Err(_) => {
            write_error("HTTP/1.1 503 Service Unavailable".to_string(), st_client)
        }
    }
}
