//! Single-file HTTP server + client using only the standard library
//! (no cargo, no serde). It shows how a client deals with JSON that has
//! null values, missing fields, wrong types, bad values and extra fields.
//!
//! Build:  rustc 43_json_server.rs --edition 2024
//! Run:    ./43_json_server server                 (terminal 1)
//!         ./43_json_server client 4               (terminal 2, lenient parsing)
//!         ./43_json_server client 5 --strict      (reject anything unexpected)
//!         ./43_json_server client all             (run every scenario)
//!         curl -i 'http://127.0.0.1:8080/user?id=4'
//!
//! Scenarios served by GET /user?id=N
//!   1  complete, correct record
//!   2  optional fields explicitly null
//!   3  optional fields missing entirely
//!   4  extra fields the client does not know about
//!   5  optional fields with the wrong type
//!   6  required field missing
//!   7  required field null
//!   8  malformed JSON (truncated body)
//!   9  right types, but invalid values (empty email, age 200)
//!   10 required field with the wrong type ("10" instead of 10)
//!   anything else -> 404, missing/empty/non-numeric id -> 400

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

const ADDR: &str = "127.0.0.1:8080";

// ============================================================================
// SERVER
// ============================================================================

fn run_server() {
    let listener = TcpListener::bind(ADDR).expect("could not bind address");
    println!("listening on http://{ADDR}  (try GET /user?id=1 .. 10)");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_connection(stream) {
                    eprintln!("connection error: {e}");
                }
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    // Read and ignore the headers; a GET has no body for us to care about.
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
    }
    drop(reader);

    let request_line = request_line.trim_end();
    let (status, body) = route(request_line);
    println!("{request_line} -> {status}");

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

/// The server validates its own input (the `id` query parameter) the same
/// way the client validates the server's output: missing, empty, wrong type.
fn route(request_line: &str) -> (&'static str, String) {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    if method != "GET" {
        return ("405 Method Not Allowed", error_json("only GET is supported"));
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != "/user" {
        return ("404 Not Found", error_json("unknown path, use /user?id=N"));
    }

    let id_param = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == "id")
        .map(|(_, value)| value);

    let id = match id_param {
        None => return ("400 Bad Request", error_json("missing query parameter 'id'")),
        Some("") => return ("400 Bad Request", error_json("query parameter 'id' is empty")),
        Some(raw) => match raw.parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                let msg = format!("'id' must be a positive integer, got '{raw}'");
                return ("400 Bad Request", error_json(&msg));
            }
        },
    };

    match user_body(id) {
        Some(body) => ("200 OK", body.to_string()),
        None => ("404 Not Found", error_json(&format!("no user with id {id}"))),
    }
}

fn user_body(id: u32) -> Option<&'static str> {
    let body = match id {
        1 => r#"{"id":1,"name":"Alice","email":"alice@example.com","age":30,"nickname":"ally"}"#,
        2 => r#"{"id":2,"name":"Bob","email":null,"age":null,"nickname":null}"#,
        3 => r#"{"id":3,"name":"Carol"}"#,
        4 => r#"{"id":4,"name":"Dave","email":"dave@example.com","age":41,"nickname":"d","role":"admin","address":{"city":"Pune","zip":"411001"},"tags":["beta","vip"],"api_version":2}"#,
        5 => r#"{"id":5,"name":"Eve","email":12345,"age":"thirty","nickname":true}"#,
        6 => r#"{"id":6,"email":"ghost@example.com","age":20}"#,
        7 => r#"{"id":7,"name":null,"email":"nobody@example.com"}"#,
        8 => r#"{"id":8,"name":"Frank","email":"frank@exa"#,
        9 => r#"{"id":9,"name":"Grace","email":"","age":200}"#,
        10 => r#"{"id":"10","name":"Heidi"}"#,
        _ => return None,
    };
    Some(body)
}

fn error_json(message: &str) -> String {
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    format!(r#"{{"error":"{escaped}"}}"#)
}

// ============================================================================
// MINIMAL JSON PARSER (what serde_json would do for you)
// ============================================================================

type Object = BTreeMap<String, Json>;

#[allow(dead_code)] // some variants' payloads are only printed via Debug
#[derive(Debug, Clone)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Object),
}

impl Json {
    fn type_name(&self) -> &'static str {
        match self {
            Json::Null => "null",
            Json::Bool(_) => "bool",
            Json::Num(_) => "number",
            Json::Str(_) => "string",
            Json::Arr(_) => "array",
            Json::Obj(_) => "object",
        }
    }
}

fn parse_json(text: &str) -> Result<Json, String> {
    let mut parser = Parser { bytes: text.as_bytes(), pos: 0 };
    let value = parser.value()?;
    parser.skip_whitespace();
    if parser.pos != parser.bytes.len() {
        return Err(parser.error("trailing characters after JSON value"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, msg: &str) -> String {
        format!("{msg} at byte {}", self.pos)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn next(&mut self) -> Result<u8, String> {
        let b = self.peek().ok_or_else(|| self.error("unexpected end of input"))?;
        self.pos += 1;
        Ok(b)
    }

    fn expect(&mut self, wanted: u8) -> Result<(), String> {
        match self.next()? {
            b if b == wanted => Ok(()),
            b => Err(self.error(&format!("expected '{}', found '{}'", wanted as char, b as char))),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_whitespace();
        match self.peek() {
            None => Err(self.error("unexpected end of input")),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(c) => Err(self.error(&format!("unexpected character '{}'", c as char))),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut map = Object::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            let value = self.value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.next()? {
                b',' => continue,
                b'}' => return Ok(Json::Obj(map)),
                c => return Err(self.error(&format!("expected ',' or '}}', found '{}'", c as char))),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_whitespace();
            match self.next()? {
                b',' => continue,
                b']' => return Ok(Json::Arr(items)),
                c => return Err(self.error(&format!("expected ',' or ']', found '{}'", c as char))),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = Vec::new();
        loop {
            match self.next()? {
                b'"' => break,
                b'\\' => match self.next()? {
                    b'"' => out.push(b'"'),
                    b'\\' => out.push(b'\\'),
                    b'/' => out.push(b'/'),
                    b'n' => out.push(b'\n'),
                    b't' => out.push(b'\t'),
                    b'r' => out.push(b'\r'),
                    b'b' => out.push(0x08),
                    b'f' => out.push(0x0c),
                    b'u' => {
                        let hex = self.bytes.get(self.pos..self.pos + 4).ok_or_else(|| self.error("bad \\u escape"))?;
                        let code = std::str::from_utf8(hex)
                            .ok()
                            .and_then(|h| u32::from_str_radix(h, 16).ok())
                            .ok_or_else(|| self.error("bad \\u escape"))?;
                        self.pos += 4;
                        let ch = char::from_u32(code).unwrap_or('\u{FFFD}');
                        out.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                    }
                    c => return Err(self.error(&format!("invalid escape '\\{}'", c as char))),
                },
                b => out.push(b),
            }
        }
        String::from_utf8(out).map_err(|_| self.error("invalid UTF-8 in string"))
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')) {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).unwrap();
        text.parse::<f64>()
            .map(Json::Num)
            .map_err(|_| self.error(&format!("invalid number '{text}'")))
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(self.error(&format!("expected '{word}'")))
        }
    }
}

// ============================================================================
// TOLERANT DECODING: JSON -> typed struct
// ============================================================================

/// Converts one JSON value into a Rust type, or explains why it can't.
trait FromJson: Sized {
    fn from_json(value: &Json) -> Result<Self, String>;
}

impl FromJson for String {
    fn from_json(value: &Json) -> Result<Self, String> {
        match value {
            Json::Str(s) => Ok(s.clone()),
            other => Err(format!("expected string, got {}", other.type_name())),
        }
    }
}

impl FromJson for u64 {
    fn from_json(value: &Json) -> Result<Self, String> {
        match value {
            Json::Num(n) if n.fract() == 0.0 && *n >= 0.0 && *n <= u64::MAX as f64 => Ok(*n as u64),
            Json::Num(n) => Err(format!("expected non-negative integer, got {n}")),
            other => Err(format!("expected number, got {}", other.type_name())),
        }
    }
}

impl FromJson for u32 {
    fn from_json(value: &Json) -> Result<Self, String> {
        let n = u64::from_json(value)?;
        u32::try_from(n).map_err(|_| format!("{n} does not fit in u32"))
    }
}

/// Option<T> can't tell "key absent" from "key: null". When that difference
/// matters (e.g. PATCH: null = "clear it", missing = "leave it alone"),
/// use a three-state type instead.
#[derive(Debug)]
enum Field<T> {
    Missing,
    Null,
    Value(T),
}

/// Decoding policy. Lenient = keep going and collect warnings (like serde's
/// default + `#[serde(default)]`). Strict = first problem is an error (like
/// `#[serde(deny_unknown_fields)]`).
struct Decoder<'a> {
    obj: &'a Object,
    strict: bool,
    warnings: Vec<String>,
}

impl<'a> Decoder<'a> {
    fn new(json: &'a Json, strict: bool) -> Result<Self, String> {
        match json {
            Json::Obj(obj) => Ok(Decoder { obj, strict, warnings: Vec::new() }),
            other => Err(format!("expected a JSON object, got {}", other.type_name())),
        }
    }

    /// A non-fatal issue: a warning in lenient mode, an error in strict mode.
    fn problem(&mut self, msg: String) -> Result<(), String> {
        if self.strict {
            Err(msg)
        } else {
            self.warnings.push(msg);
            Ok(())
        }
    }

    /// Required: missing, null or wrong type is always fatal.
    fn required<T: FromJson>(&self, key: &str) -> Result<T, String> {
        match self.obj.get(key) {
            None => Err(format!("required field '{key}' is missing")),
            Some(Json::Null) => Err(format!("required field '{key}' is null")),
            Some(v) => T::from_json(v).map_err(|e| format!("required field '{key}': {e}")),
        }
    }

    /// Optional: missing and null both become None. A wrong type is a
    /// problem (warning or error depending on policy), never a crash.
    fn optional<T: FromJson>(&mut self, key: &str) -> Result<Option<T>, String> {
        match self.obj.get(key) {
            None | Some(Json::Null) => Ok(None),
            Some(v) => match T::from_json(v) {
                Ok(x) => Ok(Some(x)),
                Err(e) => {
                    self.problem(format!("field '{key}' dropped: {e}"))?;
                    Ok(None)
                }
            },
        }
    }

    /// Like `optional`, but keeps "missing" and "null" apart.
    fn tri_state<T: FromJson>(&mut self, key: &str) -> Result<Field<T>, String> {
        match self.obj.get(key) {
            None => Ok(Field::Missing),
            Some(Json::Null) => Ok(Field::Null),
            Some(v) => match T::from_json(v) {
                Ok(x) => Ok(Field::Value(x)),
                Err(e) => {
                    self.problem(format!("field '{key}' dropped: {e}"))?;
                    Ok(Field::Missing)
                }
            },
        }
    }

    /// Extra fields are discarded simply by never reading them. Listing them
    /// is optional; strict mode turns them into an error.
    fn check_unknown(&mut self, known: &[&str]) -> Result<(), String> {
        let unknown: Vec<String> = self.obj.keys().filter(|k| !known.contains(&k.as_str())).cloned().collect();
        for key in unknown {
            let kind = self.obj[&key].type_name();
            self.problem(format!("unknown field '{key}' ({kind})"))?;
        }
        Ok(())
    }
}

#[allow(dead_code)] // fields are only printed via Debug in this demo
#[derive(Debug)]
struct User {
    id: u64,                 // required
    name: String,            // required
    email: Option<String>,   // optional: null and missing both mean None
    age: Option<u32>,        // optional, must be 0..=150
    nickname: Field<String>, // optional, but we care if it was null vs missing
}

impl User {
    const FIELDS: &'static [&'static str] = &["id", "name", "email", "age", "nickname"];

    fn from_json(json: &Json, strict: bool) -> Result<(User, Vec<String>), String> {
        let mut d = Decoder::new(json, strict)?;

        // 1. Types: is each field present and of the right shape?
        let id = d.required::<u64>("id")?;
        let name = d.required::<String>("name")?;
        let mut email = d.optional::<String>("email")?;
        let mut age = d.optional::<u32>("age")?;
        let nickname = d.tri_state::<String>("nickname")?;
        d.check_unknown(Self::FIELDS)?;

        // 2. Values: right type, but does the value make sense?
        if name.trim().is_empty() {
            return Err("required field 'name' is empty".into());
        }
        if let Some(e) = &email {
            if !e.contains('@') {
                d.problem(format!("field 'email' dropped: '{e}' is not a valid address"))?;
                email = None;
            }
        }
        if let Some(a) = age {
            if a > 150 {
                d.problem(format!("field 'age' dropped: {a} is out of range 0..=150"))?;
                age = None;
            }
        }

        Ok((User { id, name, email, age, nickname }, d.warnings))
    }
}

// ============================================================================
// CLIENT
// ============================================================================

fn http_get(path: &str) -> std::io::Result<(String, String)> {
    let mut stream = TcpStream::connect(ADDR)?;
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {ADDR}\r\nConnection: close\r\n\r\n")?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let status = head.lines().next().unwrap_or("").splitn(2, ' ').nth(1).unwrap_or("").to_string();
    Ok((status, body.to_string()))
}

fn run_client(id: Option<&str>, strict: bool) {
    let path = match id {
        Some(id) => format!("/user?id={id}"),
        None => "/user".to_string(),
    };
    println!("=== GET {path} ({}) ===", if strict { "strict" } else { "lenient" });

    let (status, body) = match http_get(&path) {
        Ok(r) => r,
        Err(e) => {
            println!("  network error: {e} (is the server running?)\n");
            return;
        }
    };
    println!("  status : {status}");
    println!("  body   : {body}");

    // Layer 1: is it JSON at all?
    let json = match parse_json(&body) {
        Ok(json) => json,
        Err(e) => {
            println!("  FAILED : body is not valid JSON: {e}\n");
            return;
        }
    };

    // Layer 2: non-2xx responses carry an error object, not a User.
    if !status.starts_with('2') {
        let msg = match &json {
            Json::Obj(obj) => obj.get("error").and_then(|v| String::from_json(v).ok()),
            _ => None,
        };
        println!("  ERROR  : {}\n", msg.unwrap_or_else(|| "server gave no error message".into()));
        return;
    }

    // Layer 3: does the JSON fit our User type?
    match User::from_json(&json, strict) {
        Ok((user, warnings)) => {
            println!("  OK     : {user:?}");
            for w in warnings {
                println!("  warn   : {w}");
            }
        }
        Err(e) => println!("  REJECT : {e}"),
    }
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let strict = args.iter().any(|a| a == "--strict");
    let rest: Vec<&str> = args.iter().map(String::as_str).filter(|a| *a != "--strict").collect();

    match rest.as_slice() {
        ["server"] => run_server(),
        ["client", "all"] => {
            for id in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "42", "abc", ""] {
                run_client(Some(id), strict);
            }
            run_client(None, strict);
        }
        ["client", id] => run_client(Some(id), strict),
        ["client"] => run_client(None, strict),
        _ => eprintln!("usage: 43_json_server server | 43_json_server client [ID|all] [--strict]"),
    }
}
