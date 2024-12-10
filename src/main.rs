use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

enum Command {
    Ping,
    Echo(String),
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(|| {
                    handle_connection(_stream);
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let mut writer = stream.try_clone().unwrap();
    let buf_reader = BufReader::new(&stream);

    // State
    let mut current_message: Vec<String> = vec![];
    let mut current_message_length: Option<usize> = None;
    let mut read_lines = 0;

    let mut lines_iterator = buf_reader.lines();

    loop {
        let line = lines_iterator.next();
        if let None = line {
            // Connection closed
            break;
        }

        let line = line.unwrap().unwrap().trim().to_string();

        if line.starts_with("*") {
            let message_length = line[1..].parse().unwrap();
            current_message_length = Some(message_length);
        } else if line.starts_with("$") {
            // Bulk string, next line contains a string, read it into current_message
            let line = lines_iterator.next().unwrap().unwrap().trim().to_string();
            current_message.push(line);
            read_lines += 1;
        }

        if let Some(msg_length) = current_message_length {
            if msg_length == read_lines {
                handle_message(&current_message, &mut writer);

                // Reset state
                current_message = vec![];
                current_message_length = None;
                read_lines = 0;
            }
        }
    }
}

fn handle_message(message: &Vec<String>, writer: &mut TcpStream) {
    let mut answer = String::new();
    match message[0].to_uppercase().as_str() {
        "PING" => {
            answer = String::from("+PONG\r\n");
        }
        "ECHO" => {
            // Echo command can have only 1 argument
            let line = &message[1];
            answer.push_str("$");
            answer.push_str(line.len().to_string().as_str());
            answer.push_str("\r\n");
            answer.push_str(line.as_str());
            answer.push_str("\r\n");
        }
        _ => panic!("No pattern found"),
    }

    writer.write_all(answer.as_bytes());
}
