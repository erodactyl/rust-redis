use anyhow::Result;
use std::{
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

enum Command {
    Ping,
    Echo(String),
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(|| {
                    handle_connection(_stream).unwrap();
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> Result<()> {
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
                let line = lines_iterator.next().unwrap().unwrap().trim().to_string();
                current_message.push(line);
                read_lines += 1;
            }

            if let Some(msg_length) = current_message_length {
                if msg_length == read_lines {
                    let command = get_command_from_message(&current_message);
                    handle_command(command, &mut writer)?;

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
        _ => panic!("No pattern found"),
    }
}

fn handle_command(command: Command, writer: &mut TcpStream) -> Result<()> {
    match command {
        Command::Ping => {
            writer.write_all(b"+PONG\r\n")?;
        }
        Command::Echo(line) => {
            writer.write_all(b"$")?;
            writer.write_all(line.len().to_string().as_bytes())?;
            writer.write_all(b"\r\n")?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
    }

    Ok(())
}
