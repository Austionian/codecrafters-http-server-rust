pub mod client {
    use anyhow::{anyhow, bail};
    use itertools::Itertools;
    use std::collections::HashMap;
    use std::fs;
    use std::str::FromStr;
    use std::string::FromUtf8Error;
    use std::sync::{Arc, Mutex};
    use tokio::io::{self, AsyncWriteExt};
    use tokio::net::TcpStream;

    #[derive(Debug)]
    pub(crate) enum Method {
        GET,
        POST,
        PUT,
    }

    #[derive(Debug)]
    pub(crate) struct StartLine {
        pub(crate) method: Method,
        pub(crate) path: String,
        pub(crate) _version: String,
    }

    #[derive(Debug)]
    pub(crate) struct Request {
        pub(crate) start_line: StartLine,
        pub(crate) headers: HashMap<String, String>,
        pub(crate) body: Option<String>,
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
                .lines()
                .next()
                .unwrap()
                .split(' ')
                .collect_tuple()
                .ok_or(anyhow!("Incorrect start line provided."))?;

            let method = method.parse()?;

            Ok(Self {
                method,
                path: path.to_string(),
                _version: version.to_string(),
            })
        }
    }

    impl FromStr for Request {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (start_line, str_headers) =
                s.split_once("\r\n").ok_or(anyhow!("Invalid request."))?;

            let start_line = start_line.parse::<StartLine>()?;

            let mut headers = HashMap::new();
            let mut body = None;

            match start_line.method {
                Method::GET => {
                    str_headers.trim_end().lines().for_each(|line| {
                        let (k, v) = line.split_once(": ").unwrap();

                        headers.insert(k.to_string(), v.to_string());
                    });
                }
                _ => {
                    let (str_headers, str_body) = str_headers.split_once("\r\n\r\n").unwrap();
                    str_headers.lines().for_each(|line| {
                        let (k, v) = line.split_once(": ").unwrap();

                        headers.insert(k.to_string(), v.to_string());
                    });

                    body = Some(str_body.to_string());
                }
            }

            Ok(Request {
                start_line,
                headers,
                body,
            })
        }
    }

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
