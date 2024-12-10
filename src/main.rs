use anyhow::Result;
use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};
use thread_safe_map::ThreadSafeMap;
mod thread_safe_map;

enum Command {
    Ping,
    Echo(String),
    Get(String),
    Set(String, String),
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    let map: ThreadSafeMap<String, String> = ThreadSafeMap::new();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let map = map.clone();
                thread::spawn(move || {
                    handle_connection(_stream, map).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(stream: TcpStream, map: ThreadSafeMap<String, String>) -> Result<()> {
    let mut writer = stream.try_clone()?;
    let buf_reader = BufReader::new(&stream);

    // State
    let mut current_message: Vec<String> = vec![];
    let mut current_message_length: Option<usize> = None;
    let mut read_lines = 0;

    let mut lines_iterator = buf_reader.lines();

    loop {
        let line = lines_iterator.next();
        if let Some(line) = line {
            let line = line?.trim().to_string();

            if line.starts_with("*") {
                let message_length = line[1..].parse()?;
                current_message_length = Some(message_length);
            } else if line.starts_with("$") {
                // Bulk string, next line contains a string, read it into current_message
                let line = lines_iterator.next().unwrap()?.trim().to_string();
                current_message.push(line);
                read_lines += 1;
            }

            if let Some(msg_length) = current_message_length {
                if msg_length == read_lines {
                    let command = get_command_from_message(&current_message);
                    handle_command(command, &mut writer, &map)?;

                    // Reset state
                    current_message = vec![];
                    current_message_length = None;
                    read_lines = 0;
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

fn get_command_from_message(message: &Vec<String>) -> Command {
    match message[0].to_uppercase().as_str() {
        "PING" => Command::Ping,
        "ECHO" => Command::Echo(message[1].clone()),
        "GET" => Command::Get(message[1].clone()),
        "SET" => Command::Set(message[1].clone(), message[2].clone()),
        _ => panic!(),
    }
}

fn handle_command(
    command: Command,
    writer: &mut TcpStream,
    map: &ThreadSafeMap<String, String>,
) -> Result<()> {
    match command {
        Command::Ping => {
            writer.write_all(b"+PONG\r\n")?;
        }
        Command::Echo(line) => {
            let response = format!("${}\r\n{line}\r\n", line.len());
            writer.write_all(response.as_bytes())?;
        }
        Command::Get(key) => {
            let value = map.get(key);
            if let Some(value) = value {
                let response = format!("${}\r\n{value}\r\n", value.len());
                writer.write_all(response.as_bytes())?;
            } else {
                writer.write_all(b"$-1\r\n")?;
            }
        }
        Command::Set(key, value) => {
            map.set(key, value);
            writer.write_all(b"+OK\r\n")?;
        }
    }

    Ok(())
}
