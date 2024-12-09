#![allow(unused_imports)]
use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut writer = stream.try_clone().unwrap();
    let buf_reader = BufReader::new(&stream);

    for line in buf_reader.lines() {
        let line = line.unwrap();
        if line == "PING" {
            writer.write_all("+PONG\r\n".as_bytes());
        }
    }
}
