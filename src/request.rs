use anyhow::{anyhow, bail};
use itertools::Itertools;
use std::collections::HashMap;
use std::str::FromStr;

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
        let (start_line, str_headers) = s.split_once("\r\n").ok_or(anyhow!("Invalid request."))?;

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
