mod request;

pub mod client {
    use crate::request::{Method, Request};
    use std::fs;
    use std::string::FromUtf8Error;
    use std::sync::{Arc, Mutex};
    use tokio::io::{self, AsyncWriteExt};
    use tokio::net::TcpStream;

    fn io_error(_: FromUtf8Error) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, "Invalid string")
    }

    pub async fn handle_client(
        mut stream: TcpStream,
        dir: Arc<Mutex<Option<String>>>,
    ) -> Result<usize, io::Error> {
        let mut buf = Vec::with_capacity(4096);

        let _ = stream.readable().await;

        let _ = stream.try_read_buf(&mut buf)?;

        let request = String::from_utf8(buf).map_err(io_error)?.parse::<Request>();

        return match request {
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
                _ if s.start_line.path.starts_with("/files") => match s.start_line.method {
                    Method::GET => {
                        let file_name = s.start_line.path.split_once("/files/").unwrap().1;
                        let dir_name = dir.lock().unwrap().clone().unwrap();
                        let file =
                            fs::read_to_string(format!("{}{}", dir_name, file_name).as_str());

                        match file {
                    Ok(f) => {
                        stream
                            .write(
                                format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}\r\n",
                        f.len(),
                        f
                    )
                                .as_bytes(),
                            )
                            .await
                    }
                    Err(_) => stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await,
                }
                    }
                    Method::POST => {
                        let file_name = s.start_line.path.split_once("/files/").unwrap().1;
                        let dir_name = dir.lock().unwrap().clone().unwrap();
                        let _ = fs::write(format!("{}{}", dir_name, file_name), s.body.unwrap());

                        stream
                            .write(format!("HTTP/1.1 201 OK\r\n\r\n",).as_bytes())
                            .await
                    }
                    Method::PUT => unimplemented!(),
                },
                _ => stream.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await,
            },
            Err(_) => stream.write(b"HTTP/1.1 500 Internal Server Error").await,
        };
    }
}
