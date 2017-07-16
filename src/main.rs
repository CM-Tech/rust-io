mod hash;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use hash::hash_key;

fn home_page() -> String {
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}",
            include_str!("../static/index.html"))
}

fn handle_websocket(mut stream: TcpStream) {
    loop {
        let mut bytes = vec![0; 512];
        match stream.read(&mut bytes) {
            Ok(len) if len > 0 => {
                bytes.truncate(len);

                if bytes[0] == 0x81 {
                    let index_first_mask = 2;
                    let index_first_data_byte = index_first_mask + 4;
                    let masks = &bytes[index_first_mask..index_first_data_byte];

                    let mut dec = vec![0; len - index_first_data_byte];
                    for j in 0...(len - index_first_data_byte){
                        
                        dec[j] = bytes[j + index_first_data_byte] ^ masks[j % 4];
                    }
                    println!("{:?}", String::from_utf8(dec));

                    let string = "Hello".as_bytes();
                    let data = [0x81, string.len() as u8];
                    let mut v = vec![];
                    v.extend_from_slice(&data);
                    v.extend_from_slice(&string);
                    stream.write(&v).ok();
                    stream.flush().ok();
                }
            }
            Err(e) => {
                println!("closing connection because: {}", e);
                return;
            }
            _ => (),
        };
    }
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
                (None, _) if req_url == "/" => {
                    stream.write(home_page().as_bytes()).ok();
                    stream.flush().ok();

                    return;
                }
                (Some(_), Some(key)) => {
                    let key_value = key.split(": ").collect::<Vec<&str>>()[1];
                    let accept_header = format!("Sec-WebSocket-Accept: {}", hash_key(key_value.as_bytes()));
                    let strings: Vec<&str> = vec!["HTTP/1.1 101",
                                                  "Connection: Upgrade",
                                                  "Upgrade: websocket",
                                                  accept_header.as_str(),
                                                  "\r\n"];
                    stream.write(strings.join("\r\n").as_bytes()).ok();
                    stream.flush().ok();

                    handle_websocket(stream)
                }
                _ => {
                    println!("{}", req_url);

                    return;
                }
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
