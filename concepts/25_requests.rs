use std::io::{Read, Write};
use std::net::TcpStream;


fn main() -> std::io::Result<()> {
    
    for i in 1..=11 {
        let mut stream = TcpStream::connect("127.0.0.1:8080")?;
        let request = format!(
            "GET /user?id={} HTTP/1.1\r\n\
             Host: 127.0.0.1:8080\r\n\
             Connection: close\r\n\
             \r\n",
            i
        );
        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        println!("{}", response);
    }

    Ok(()) //Ok(()) means the operation succeeded, and there is no meaningful value to return.
}