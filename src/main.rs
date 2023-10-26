// Uncomment this block to pass the first stage
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

fn handle_client(mut stream: TcpStream) -> Result<(), io::Error> {
    let mut buffer = String::new();
    let _ = stream.read_to_string(&mut buffer)?;

    println!("{buffer}");

    let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
    stream.shutdown(Shutdown::Write)?;

    Ok(())
}

fn main() -> Result<(), io::Error> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream)?,
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
