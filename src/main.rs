use anyhow::{anyhow, bail};
use itertools::Itertools;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

#[derive(Debug)]
enum Method {
    GET,
    POST,
    PUT,
}

#[derive(Debug)]
struct StartLine {
    _method: Method,
    path: String,
    _version: String,
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            _ => bail!("Invalid method given."),
        }
    }
}

impl FromStr for StartLine {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (method, path, version) = s
            .split(' ')
            .collect_tuple()
            .ok_or(anyhow!("Incorrect start line provided."))?;

        let _method = method.parse()?;

        Ok(StartLine {
            _method,
            path: path.to_string(),
            _version: version.to_string(),
        })
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = String::new();
    let _ = stream
        .read_to_string(&mut buffer)
        .expect("unable to read to buffer");

    let start_line = buffer.parse::<StartLine>();

    match start_line {
        Ok(s) => {
            if s.path == "/" {
                stream
                    .write(b"HTTP/1.1 200 OK\r\n\r\n")
                    .expect("Failed to write to stream.");
            } else {
                stream
                    .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                    .expect("Failed to write to stream.");
            }
        }
        Err(_) => {
            stream
                .write(b"HTTP/1.1 500 Internal Server Error")
                .expect("Failed to write to stream.");
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
