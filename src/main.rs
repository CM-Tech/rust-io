mod hash;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use hash::hash_key;

fn home_page() -> String {
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}",
            include_str!("../static/index.html"))
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 512];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            let mut headers = req_str.lines();
            let req_url = headers.next().unwrap().split(" ").collect::<Vec<&str>>()[1];
            let upgrade = headers.find(|x| x.contains("Upgrade:"));
            let key = headers.find(|x| x.contains("Sec-WebSocket-Key:"));

            match (upgrade, key) {
                (None, _) => {
                    println!("{}", req_url);
                    if req_url == "/" {
                        stream.write(home_page().as_bytes());
                        stream.flush();
                    }
                    return;
                }
                (Some(_), Some(key)) => {
                    let key_value = key.split(": ").collect::<Vec<&str>>()[1];
                    let bla = format!("Sec-WebSocket-Accept: {}",
                                      hash_key(key_value.as_bytes()));
                    let strings: Vec<&str> = vec!["HTTP/1.1 101",
                                                  "Connection: Upgrade",
                                                  "Upgrade: websocket",
                                                  bla.as_str(),
                                                  "\r\n"];
                    stream.write(strings.join("\r\n").as_bytes());
                    stream.flush();
                }
                _ => (),
            }
        }
        Err(e) => println!("Unable to read stream: {}", e),
    }
}

fn main() {
    let addr = "127.0.0.1:8888";
    let listener = TcpListener::bind(addr);
    match listener {
        Ok(listen) => {
            println!("Listening on addr: {}", addr);
            for stream in listen.incoming() {
                let stream = stream.unwrap();
                thread::spawn(move || handle_client(stream));
            }
        }
        Err(e) => println!("Failed to bind server: {}", e),
    }
}
