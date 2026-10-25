// A tiny HTTP client: sends 11 requests to 24_json_server.rs
//
// Run the server first (terminal 1):  rustc 24_json_server.rs --edition 2024 && ./24_json_server server
// Then this client (terminal 2):      ./rs.sh 25_requests.rs
//
// HTTP is just text over a TCP connection. A GET request looks like:
//
//   GET /user?id=1 HTTP/1.1      <- method, path, version
//   Host: 127.0.0.1:8080         <- headers, one per line
//   Connection: close
//                                <- empty line = end of headers
//
// Lines end with \r\n (carriage return + newline), as HTTP requires.
// ids 1-10 are the server's test scenarios; id 11 doesn't exist, so it gets a 404.
//
// | Step                     | Code                          |
// |--------------------------|-------------------------------|
// | open a connection        | TcpStream::connect(addr)?     |
// | send the request text    | stream.write_all(bytes)?      |
// | read the whole reply     | stream.read_to_string(&mut s)?|

// Read / Write are traits (see 15_traits.rs). They must be in scope to call
// read_to_string() and write_all() on the stream.
use std::io::{Read, Write};
use std::net::TcpStream;


// main can return a Result too. std::io::Result<()> means "Ok(()) or an I/O error".
// That lets us use `?`: on Err, main stops and Rust prints the error.
fn main() -> std::io::Result<()> {
    
    for i in 1..=11 {
        // A new connection per request, because the server closes it after replying.
        let mut stream = TcpStream::connect("127.0.0.1:8080")?;
        // format! builds a String like println!, without printing it.
        // A `\` at the end of a line inside a string skips the newline AND the
        // next line's leading spaces, so this is one string with no indentation.
        // "Connection: close" asks the server to close the connection when done,
        // which is how read_to_string below knows the reply has ended.
        let request = format!(
            "GET /user?id={} HTTP/1.1\r\n\
             Host: 127.0.0.1:8080\r\n\
             Connection: close\r\n\
             \r\n",
            i
        );
        // Network sends bytes, not text: as_bytes() gives the String's bytes.
        stream.write_all(request.as_bytes())?;

        let mut response = String::new();
        // Reads until the server closes the connection.
        stream.read_to_string(&mut response)?;

        // Prints the status line, headers, and JSON body.
        println!("{}", response);
    }

    Ok(()) //Ok(()) means the operation succeeded, and there is no meaningful value to return.
}
