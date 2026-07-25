use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("failed to bind to 127.0.0.1:7878");

    println!("listening on 127.0.0.1:7878");

    loop {
        handle_connection(&listener);
    }
}

fn handle_connection(listener: &TcpListener) {
    let (mut stream, addr) = listener
        .accept()
        .expect("failed to accept connection");

    println!("accepted connection from {addr}");

    let mut buffer = [0u8; 1024];
    let bytes_read = stream
        .read(&mut buffer)
        .expect("failed to read from stream");

    println!("--- received {bytes_read} bytes ---");
    println!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
    println!("--- end ---");

    let body = "<b>hello from server</b>\n";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        body.len(),
        body
    );

    stream
        .write_all(response.as_bytes())
        .expect("failed to write to stream");
}
