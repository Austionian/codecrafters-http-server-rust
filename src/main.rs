use anyhow::{anyhow, bail};
use itertools::Itertools;
use std::collections::HashMap;
use std::str::FromStr;
use std::string::FromUtf8Error;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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

#[derive(Debug)]
struct Request {
    start_line: StartLine,
    headers: HashMap<String, String>,
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

        Ok(Self {
            _method,
            path: path.to_string(),
            _version: version.to_string(),
        })
    }
}

impl FromStr for Request {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let request = s.split_once("\n\n");

        return match request {
            Some((start_line, str_headers)) => {
                let start_line = start_line.parse::<StartLine>()?;

                let mut headers = HashMap::new();

                str_headers.lines().for_each(|line| {
                    let (k, v) = line.split_once(": ").unwrap();

                    headers.insert(k.to_string(), v.to_string());
                });

                Ok(Request {
                    start_line,
                    headers,
                })
            }
            None => {
                let start_line = s.parse::<StartLine>()?;

                Ok(Request {
                    start_line,
                    headers: HashMap::default(),
                })
            }
        };
    }
}

fn io_error(_: FromUtf8Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Invalid string")
}

async fn handle_client(mut stream: TcpStream) -> Result<usize, io::Error> {
    let mut buf = Vec::with_capacity(4096);

    let _ = stream.readable().await;

    let _ = stream.try_read_buf(&mut buf)?;

    let start_line = String::from_utf8(buf).map_err(io_error)?.parse::<Request>();

    return match start_line {
        Ok(s) => match s.start_line.path.as_str() {
            "/" => stream.write(b"HTTP/1.1 200 OK\r\n\r\n").await,
            _ if s.start_line.path.starts_with("/echo") => {
                let path = s
                    .start_line
                    .path
                    .split_once("/echo/")
                    .expect("Invalid path provided")
                    .1;
                return stream.write(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
                        path.len(),
                        path
                    ).as_bytes()).await;
            }
            _ if s.start_line.path.starts_with("/user-agent") => {
                let user_agent = s.headers.get("User-Agent").ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "No user agent header.",
                ))?;
                return stream.write(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n",
                        user_agent.len(),
                        user_agent
                    ).as_bytes()).await;
            }
            _ => stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await,
        },
        Err(_) => stream.write(b"HTTP/1.1 500 Internal Server Error").await,
    };
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        handle_client(socket).await?;
    }
}
