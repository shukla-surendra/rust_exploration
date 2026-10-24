# Custom Web Server in Rust — Build Plan

## How to use this document

This is a milestone-based plan, not code. Each milestone lists what you should
be able to *do* by the end of it, a checklist of tasks, what to look up in the
docs, and how to verify it actually works (usually with `curl`). Work through
them in order — each one depends on the last. Don't skip ahead to routing
before raw parsing works, or to concurrency before a single connection works
correctly.

Check things off as you go. When you get stuck, look up the specific std
module in the docs before reaching for a crate — the point of this project is
to understand what's normally hidden by a framework.

**Approach**: synchronous, no async runtime. `std::net::TcpListener`, one
thread per connection (later a thread pool), hand-rolled HTTP/1.1 parsing.

**Scope**: static file server with basic path/method routing, plus an
optional reverse-proxy mode that forwards requests to a backend application
(e.g. a Flask API). No keep-alive, no chunked encoding, no TLS — those are
listed as stretch goals at the end if you want to keep going after the core
is done.

---

## Rust concepts cheat sheet

A quick map of *why* each Rust feature shows up in this project, so the
milestones below make sense before you hit them. Look these up as you reach
the milestone that needs them — this is just an index.

| Concept | Why you need it here |
|---|---|
| `std::net::TcpListener` / `TcpStream` | The actual sockets. A `TcpListener` is the thing bound to a port; each accepted connection gives you a `TcpStream`, which is both `Read` and `Write`. |
| `Read` / `Write` traits | `TcpStream` doesn't have its own `.read_line()` etc. — you use these traits' methods (`read`, `read_to_end`) or wrap the stream in a `BufReader` to get line-buffered reads. |
| `io::Result<T>` / `?` operator | Almost everything socket/file related returns `io::Result`. Propagate with `?` inside functions that return `Result`; at the top of a per-connection handler, match/log instead of propagating further, so one bad connection can't unwind the whole server. |
| Ownership of `TcpStream` | Each accepted `TcpStream` is owned by whichever thread handles it. Passing it into `thread::spawn(move || ...)` transfers ownership into the thread — this is *why* threading a connection is natural in Rust: no shared mutable socket state to worry about. |
| `std::thread::spawn` | Gives you real OS-level parallelism (no GIL, unlike Python) — this is literally how multiple requests get handled "at the same time." Closure must be `'static + Send`. |
| `Arc<T>` | Share read-only data (e.g. the static file root path, a route table) across threads without cloning the underlying data — clones of the `Arc` share one heap allocation via atomic refcounting. |
| `Mutex<T>` | Share *mutable* state across threads safely (e.g. a job queue, a request counter) — only needed once something is written to, not just read, from multiple threads. |
| `mpsc::channel` | The producer/consumer queue behind a thread pool: the accept loop sends `Job`s in, worker threads receive and run them. |
| `Box<dyn FnOnce() + Send + 'static>` | The type you need for "a job to run on some worker thread" when building a thread pool — a boxed, type-erased closure. |
| Enums for `Method` / `ParseError` / status codes | Rust's enums make illegal states unrepresentable — e.g. an invalid method literally can't reach your routing logic if parsing rejects it first. |
| `impl Display` / `impl std::error::Error` | Standard way to make your own error types printable and composable with `?`. |

---

## Milestone 0 — Project setup & a raw TCP echo

**Goal**: prove you can accept a raw TCP connection and read/write bytes on
it, with zero HTTP involved yet.

- [ ] Add nothing to `Cargo.toml` yet — pure `std` for this milestone.
- [ ] Open a `TcpListener` bound to `127.0.0.1:7878` (or a port of your choice).
- [ ] Accept one connection, read whatever bytes the client sends, print them
      as a lossy UTF-8 string.
- [ ] Write a fixed string back to the client and close the connection.

**Docs to read**: `std::net::TcpListener`, `std::net::TcpStream`,
`std::io::Read`, `std::io::Write`.

**Rust hints**:
- `TcpListener::bind(addr)` returns `io::Result<TcpListener>` — the `bind`
  can fail (e.g. port already in use), so this is your first real `?` or
  `.unwrap()` decision. In `main`, failing to bind is fatal, so `.expect(...)`
  with a clear message is reasonable here (this is one of the few places in
  the whole project where an early `.unwrap()`/`.expect()` is fine — it's
  during startup, not while serving a request).
- A single accepted connection: `listener.accept()` gives you
  `io::Result<(TcpStream, SocketAddr)>`.
- `TcpStream` implements `Read` and `Write` directly, but for reading text
  a byte at a time or a line at a time, wrap it:
  `let mut reader = BufReader::new(&stream);` then use
  `reader.read_line(&mut String::new())` or `read_until`.
- Reading raw bytes without assuming they're valid UTF-8: read into a
  `Vec<u8>` or `[u8; N]` buffer with `.read()`, then
  `String::from_utf8_lossy(&buffer)` just for printing/debugging.

**Verify it**: `nc 127.0.0.1 7878` and type some text, or
`curl -v http://127.0.0.1:7878/` and look at what bytes arrived in your
program's stdout — this is your first look at what a raw HTTP request looks
like on the wire.

---

## Milestone 1 — Loop and accept multiple connections (still single-threaded)

**Goal**: the server doesn't die after one request.

- [ ] Wrap the accept step in a loop so the server keeps handling new
      connections, one at a time, forever.
- [ ] Extract the per-connection logic into its own function, e.g.
      `handle_connection(stream: TcpStream)`.
- [ ] Make sure a connection that errors (e.g. client disconnects abruptly)
      doesn't crash the whole server — handle the `Result` instead of
      `.unwrap()`-ing where it matters.

**Rust hints**:
- This loop is *the* answer to "how does the server stay up": `for stream in
  listener.incoming() { ... }` (or an explicit `loop { let (stream, _) =
  listener.accept()?; ... }`) blocks on `accept()` each iteration — the OS
  parks the thread until a client connects. The process never exits on its
  own because this loop has no `break`; it only exits on a signal (Ctrl-C) or
  a fatal unhandled error. This is the entire mechanism — no special
  "keep-alive daemon" logic is needed for the process itself to stay running.
- `listener.incoming()` yields `io::Result<TcpStream>` items — a `Result` per
  *connection attempt*, not per byte. If one `Err` comes through (rare, but
  possible), `continue` the loop rather than propagating it out of the
  overall server loop.
- Inside `handle_connection`, any error should be logged (`eprintln!` is
  fine for now) and the function should just `return`, dropping the
  `TcpStream` — Rust closes the socket automatically via `Drop` when it goes
  out of scope. You don't need to manually call a "close" method.

**Verify it**: run several `curl` requests back to back against the server
without restarting it.

---

## Milestone 2 — Parse the HTTP request line and headers

**Goal**: turn raw bytes into a structured `Request`.

- [ ] Design a `Request` struct: method, path, HTTP version, headers (some
      kind of map), and body (bytes, possibly empty).
- [ ] Design a `Method` enum: at minimum `GET`, `POST`, maybe `PUT`/`DELETE`
      later. Decide what happens for a method you don't recognize.
- [ ] Parse the request line (`GET /path HTTP/1.1`) — split on spaces,
      validate there are exactly 3 parts, reject with an error type if not.
- [ ] Parse headers line by line until you hit an empty line (`\r\n\r\n` marks
      the end of headers). Watch out for `\r\n` vs `\n` line endings —
      real clients send `\r\n`.
- [ ] Use `Content-Length` (if present) to know how many more bytes to read
      for the body. Don't try to read the body if there isn't one.
- [ ] Decide on your error type now (e.g. `enum ParseError { ... }`) rather
      than panicking on malformed input — you'll need this for Milestone 6
      anyway.

**Docs to read**: `BufReader`, `std::io::BufRead::read_line`, `str::split`,
`str::splitn`.

**Rust hints**:
- Read the request line and headers with `BufReader::new(stream).lines()`
  (an iterator of `io::Result<String>`, already split on `\n` with the
  trailing newline stripped) *or* manually with `read_line` in a loop —
  either works, `lines()` is less code. Stop consuming header lines the
  moment you get an empty string (that's the blank line).
- Careful: `BufReader::lines()` strips `\n` but not a trailing `\r` — trim it
  yourself (`line.trim_end_matches('\r')`) or you'll get headers with
  invisible `\r` stuck in the value.
- Parse the request line with `line.splitn(3, ' ')` → collect into a
  `Vec<&str>` or destructure via `.next()` three times; reject if you don't
  get exactly method/path/version.
- Parse each header line by splitting once on `": "` —
  `line.split_once(": ")` returns `Option<(&str, &str)>`, exactly what you
  want here.
- Store headers in a `HashMap<String, String>` (or a `Vec<(String, String)>`
  if you want to preserve order/duplicates — HTTP technically allows repeated
  headers). Lowercase the header names when storing/looking up, since HTTP
  header names are case-insensitive but Rust string comparison isn't.
- Once headers are parsed, look up `Content-Length` via
  `headers.get("content-length")`, parse it with `.parse::<usize>()`, then
  read exactly that many more bytes from the same `BufReader` with
  `read_exact(&mut buf)` — not `read_to_end`, which would block waiting for
  the connection to close (it won't, until you respond).
- For the error type, `enum ParseError { MalformedRequestLine, InvalidMethod,
  MissingHeaders, InvalidContentLength, Io(std::io::Error) }` (adjust to
  taste) is enough — implement `From<std::io::Error> for ParseError` so `?`
  works inside your parsing function even though it mixes IO and parse
  errors.

**Verify it**: send requests with `curl -v`, with extra headers
(`curl -H "X-Foo: bar" ...`), and with a body (`curl -d "hello" ...`), and
confirm your parsed struct matches what was sent. Also try something
deliberately malformed with `nc` (e.g. garbage first line) and confirm your
parser returns an error instead of panicking.

---

## Milestone 3 — Send a real HTTP response

**Goal**: a browser can load a page from your server.

- [ ] Design a `Response` struct: status code, status text, headers, body.
- [ ] Write the serialization: status line, headers, blank line, body —
      remembering `\r\n` line endings and the required blank line before the
      body.
- [ ] Always set `Content-Length` on responses that have a body.
- [ ] Hardcode a single `200 OK` response with an HTML body for now,
      regardless of what was requested.

**Rust hints**:
- Serialize into a single `String` (or `Vec<u8>` if the body might be binary
  — safer long-term, since files in Milestone 4 aren't all text) building it
  with `format!` or repeated `write!`/`writeln!` calls, then
  `stream.write_all(bytes)`. Note the header block uses `\r\n` line endings
  and the header section ends with an *extra* blank `\r\n` before the body
  — a single missing `\r\n` here is the most common bug at this stage.
- If body is `Vec<u8>`, don't try to `format!` it into the same string as the
  headers (that requires it to be valid UTF-8, which a binary file isn't).
  Instead write the header block as a `String`/`&str` via `write_all`, then
  write the body bytes via a separate `write_all` call.
- `stream.flush()` after writing isn't strictly required for `TcpStream`
  (writes go straight to the socket) but it's good habit and free.
- Status line format: `HTTP/1.1 200 OK\r\n` — a small `match status_code {
  200 => "OK", 404 => "Not Found", ... }` (or an enum, see Milestone 6/7) is
  enough for the reason phrase.

**Verify it**: point an actual browser at `http://127.0.0.1:7878/` and
confirm the page renders. Also check `curl -v` output to confirm headers look
correct (matching `Content-Length` to actual body size is a common bug here).

---

## Milestone 4 — Serve static files from disk

**Goal**: the server becomes an actual static file server.

- [ ] Pick (or create) a directory of static files to serve, e.g. a `public/`
      or `www/` folder with an `index.html` and a couple of other files.
- [ ] Map the request path to a file path under that root directory.
      `/` should map to `index.html`.
- [ ] **Security**: guard against path traversal (`/../../etc/passwd`).
      Canonicalize the resolved path and confirm it's still inside the root
      directory before reading it. Don't skip this — it's the classic
      static-file-server bug.
- [ ] Return `404 Not Found` (with a body) when the file doesn't exist.
- [ ] Set `Content-Type` based on file extension (at least: html, css, js,
      png, jpg, plain text fallback). A small extension→mime-type match is
      fine — no crate needed for this scope.
- [ ] Return `500 Internal Server Error` for unexpected IO errors (permission
      denied, etc.) rather than crashing.

**Docs to read**: `std::fs::read`, `std::path::Path::canonicalize`,
`std::path::Path::starts_with`.

**Rust hints**:
- Join the request path onto the root with `Path::new(root).join(
  requested_path.trim_start_matches('/'))` — joining an *absolute* path
  (leading `/`) onto another path with `.join()` silently discards the root
  and takes the absolute one instead, so strip that leading slash first, or
  you've reopened the traversal hole a different way.
- `path.canonicalize()` returns `io::Result<PathBuf>` and resolves `..` and
  symlinks — but it errors if the path doesn't exist, so canonicalize the
  *root* once at startup, and canonicalize the *requested* file only after
  confirming with `.exists()` (or just handle the `Err` as "file not found").
- The actual traversal guard:
  `resolved_path.starts_with(&canonical_root)` — if false, treat it exactly
  like a 404 (don't leak "path traversal detected" to the client — just a
  normal not-found response).
- `std::fs::read(path)` gives `io::Result<Vec<u8>>` directly — use this
  rather than opening a `File` and reading it yourself for this scope.
- For content-type, match on
  `path.extension().and_then(|e| e.to_str())` against a handful of `&str`
  cases; anything unrecognized can fall back to
  `application/octet-stream`.
- Distinguish `io::ErrorKind::NotFound` (→ 404) from other IO errors
  (permission denied, etc. → 500) by matching on `err.kind()`.

**Verify it**: fetch multiple existing files, a nonexistent one, and try a
path traversal attempt with `curl` (e.g.
`curl "http://127.0.0.1:7878/../Cargo.toml"` and a raw `%2e%2e` variant) —
confirm all traversal attempts are rejected.

---

## Milestone 5 — Concurrency

**Goal**: one slow client can't block others.

- [ ] First pass: spawn a new OS thread per connection
      (`std::thread::spawn`). Simple, and fine for this project's scale.
- [ ] Confirm two concurrent slow requests don't block each other — you can
      simulate "slow" with a deliberate `thread::sleep` in one handler path,
      or a large file transfer, and fire two `curl`s at once from separate
      terminals.
- [ ] Stretch: replace the naive thread-per-connection with a fixed-size
      thread pool (bounded number of worker threads pulling jobs off a
      channel) so you're not spawning unbounded threads under load. This is
      the same exercise as the multithreaded web server chapter in the Rust
      book, if you want a reference after attempting it yourself.

**Docs to read**: `std::thread::spawn`, `std::sync::mpsc`, `Arc`, `Mutex`.

**Rust hints — how "simultaneous" requests actually happen**:
- Concretely: your Milestone 1 loop currently does
  `accept()` → `handle_connection(stream)` → back to `accept()`, all on one
  thread. That means request #2 can't even be *accepted* until request #1's
  handler function returns — this is the bug you're fixing here.
- The fix: `let (stream, _) = listener.accept()?; thread::spawn(move ||
  handle_connection(stream));` — the accept loop immediately goes back to
  waiting for the next connection, while the spawned OS thread handles the
  previous one independently. The OS scheduler then genuinely runs multiple
  threads in parallel across CPU cores — this is real parallelism, not
  cooperative/single-core concurrency (unlike, say, Python's default
  threading under the GIL, or JS's single-threaded event loop).
- `thread::spawn` requires the closure to be `'static` (own everything it
  captures, no borrowed references to short-lived stack data) and `Send`
  (the captured data must be safe to move to another thread) — this is
  *why* you `move` the `stream` into the closure rather than borrow it: the
  original loop's stack frame will move on to accept the next connection
  immediately, so the thread can't be borrowing from it.
- `thread::spawn` returns a `JoinHandle<T>`. For this design you generally
  *don't* call `.join()` on it (that would block the accept loop waiting for
  the request to finish — the opposite of what you want). Dropping the
  handle detaches the thread to run independently — that's intentional here.
- Thread pool (the stretch part of this milestone): the standard shape is
  `type Job = Box<dyn FnOnce() + Send + 'static>;` sent over
  `mpsc::channel::<Job>()`. Multiple worker threads each hold a clone of
  `Arc<Mutex<mpsc::Receiver<Job>>>` (the `Mutex` because `Receiver` isn't
  `Sync` — only one thread can be pulling the next job at a time; the `Arc`
  so all workers can share ownership of that single receiver). Each worker
  loops: lock the mutex, `.recv()` a job, drop the lock, run the job. The
  accept loop just does `sender.send(Box::new(move || handle_connection(
  stream))).unwrap()` instead of `thread::spawn` directly.
- If you don't share any mutable state across connections in this milestone,
  you may not need `Mutex` for anything *except* the thread pool's job
  queue — that's expected and fine.

**Verify it**: use a tool like `ab` (ApacheBench) or a handful of parallel
`curl`s to confirm the server handles concurrent load without one request
stalling another.

---

## Milestone 6 — Routing

**Goal**: different paths/methods do different things, not just "serve a
file."

- [ ] Design a routing mechanism: a table/list mapping (method, path pattern)
      → handler. Start with exact-match paths; add simple pattern support
      (e.g. `/users/:id`) only if you want it.
- [ ] Add at least one non-static-file route, e.g. `GET /health` returning
      `200 OK` with a small JSON or text body, to prove routing is decoupled
      from the file server.
- [ ] Fall back to the static file handler for anything not matched by an
      explicit route (or vice versa — decide the precedence and document it
      to yourself).
- [ ] Return `405 Method Not Allowed` for a path that exists under a
      different method, and `404` for paths that don't exist at all.

**Rust hints**:
- A route table can be as simple as
  `Vec<(Method, &'static str, HandlerFn)>` where
  `type HandlerFn = fn(&Request) -> Response;` for exact-match routes — plain
  function pointers are enough here, you don't need trait objects or
  closures unless a handler needs to capture state (in which case
  `Box<dyn Fn(&Request) -> Response + Send + Sync>` is the closure-capable
  equivalent).
- If you want `:id`-style params without a regex crate: split both the
  route pattern and the request path on `/` into segments, compare segment
  by segment — a pattern segment starting with `:` matches anything and
  captures it into a `HashMap<String, String>` you attach to the request
  before calling the handler.
- Precedence logic is just an `if`/`else` (or early return) in your
  dispatcher: check the explicit route table first; if nothing matches, try
  the static file handler; if that 404s too, return your own 404 — don't let
  the static-file handler's 404 respond, since you may want a consistent
  error page function shared by both fall-through paths.

**Verify it**: hit the same path with different methods
(`curl -X GET`, `curl -X POST`, `curl -X DELETE`) and confirm status codes
match your routing rules.

---

## Milestone 7 — Robustness pass

**Goal**: the server doesn't fall over on real-world garbage input.

- [ ] Malformed request line → `400 Bad Request` instead of a panic.
- [ ] Missing/invalid `Content-Length` with a body present → handle
      gracefully.
- [ ] A client that connects and sends nothing (or sends data very slowly) —
      decide on a read timeout so a thread doesn't hang forever
      (`TcpStream::set_read_timeout`).
- [ ] An oversized request (very long header block, huge declared
      `Content-Length`) → reject before exhausting memory. Pick and enforce a
      max size.
- [ ] Sweep all remaining `.unwrap()`/`.expect()` calls in request-handling
      paths and confirm each one is either truly infallible or replaced with
      proper error handling.

**Rust hints**:
- `stream.set_read_timeout(Some(Duration::from_secs(10)))` — after this, a
  blocked `read` call returns an `Err` with `ErrorKind::WouldBlock` /
  `TimedOut` instead of hanging forever; treat that as "close the
  connection," not a panic.
- Enforce max sizes *while parsing*, not after: e.g. if you're reading
  headers line-by-line, count total bytes consumed so far and bail out with
  a `400`/close once you cross your limit, rather than reading everything
  first and checking length afterward (that defeats the point — the memory
  is already used).
- A `catch_unwind` around the whole per-connection handler is a reasonable
  last-resort safety net (`std::panic::catch_unwind`) so a bug in one
  request's handling can't take down other in-flight threads — but treat it
  as a backstop, not a substitute for actually handling the `Result`s above.

**Verify it**: throw deliberately broken input at the server with `nc` (raw
garbage, a request with no terminating blank line, an absurdly long header)
and confirm it responds with an error status or closes the connection
cleanly — never panics or hangs the whole process.

---

## Milestone 8 — Reverse proxy: hand requests off to a backend app (e.g. Flask)

**Goal**: your Rust server stops answering some requests itself and instead
forwards them to a separate backend process (e.g. a Python Flask app running
on `127.0.0.1:5000`), relaying its response back to the original client. This
is the same architectural role as nginx/Apache sitting in front of
Gunicorn/uWSGI/the Flask dev server — you're building a minimal version of
that.

**Why this is a separate milestone, not a handler**: the earlier
`static_files`/route handlers respond using data your process already has
(bytes on disk, a hardcoded string). A proxy handler instead has to act as an
**HTTP client** itself — open its *own* outbound TCP connection to the
backend, send a request, read a response — while still being the *server*
from the original client's point of view. Same `Request`/`Response` types
from Milestones 2–3, used in both directions.

- [ ] Start a real backend to test against — a two-route Flask app
      (`pip install flask`, `flask run --port 5000`) with e.g.
      `GET /api/hello` and `POST /api/echo` is enough. This can live in a
      sibling directory; it's scaffolding for testing, not part of the Rust
      project.
- [ ] Add a proxy rule to your router (Milestone 6): requests under some
      prefix (e.g. `/api/*`) go to the proxy handler instead of the static
      file handler or an in-process route.
- [ ] In the proxy handler: open `TcpStream::connect("127.0.0.1:5000")` to
      the backend.
- [ ] **Rebuild the outbound request** from the parsed `Request` rather than
      blindly replaying the original raw bytes — this is where you rewrite:
  - the `Host` header to the backend's host:port, since the original
    `Host` was your Rust server's address, not the backend's;
  - strip **hop-by-hop headers** that only make sense between your original
    client and *your* server, not between you and the backend:
    `Connection`, `Keep-Alive`, `Proxy-Connection`, `Transfer-Encoding`,
    `Upgrade`. (This distinction — hop-by-hop vs end-to-end headers — is a
    real HTTP/1.1 concept, not something proxies invented ad hoc.)
  - optionally add `X-Forwarded-For` (original client IP) and
    `X-Forwarded-Host` (original `Host` value) — the conventional way a
    backend app finds out about the real client despite the request coming
    from your proxy's IP. Flask can read these via `request.headers`.
- [ ] Send the rebuilt request to the backend, then **read its response
      fully** — this means running your own HTTP response parser (mirror
      image of your `Request` parser: status line, headers, then
      `Content-Length` bytes of body) against what the backend sends back.
- [ ] Relay that response back to the original client: same status code,
      same body; you can pass most headers through as-is, but strip the same
      hop-by-hop set again on the way back.
- [ ] Handle backend failure explicitly: if `TcpStream::connect` to the
      backend fails (backend not running, refused connection), respond to
      the original client with `502 Bad Gateway` — don't let that error
      propagate as a crash or hang.
- [ ] Set a connect/read timeout to the backend (same
      `set_read_timeout`/`set_write_timeout` idea as Milestone 7) and return
      `504 Gateway Timeout` if the backend takes too long to respond.

**Rust hints**:
- The outbound connection to the backend uses the exact same
  `TcpStream`/`Read`/`Write` APIs as the inbound one — the only new idea is
  that *your* code is now on the client side of a connection instead of the
  server side. If your `Request`/`Response` serialization from Milestones
  2–3 is written generically, you can reuse it here directly: serialize a
  `Request` to bytes to send to the backend, and parse bytes from the
  backend back into a `Response`.
- Don't try to stream bytes through byte-by-byte as they arrive from the
  client and forward them live to the backend for this scope — buffer the
  full request, then the full response. True streaming proxying (useful for
  large uploads/downloads or long-lived connections) is real added
  complexity; treat it as a stretch goal, not part of this milestone.
- This handler runs on the same per-connection thread as everything else
  (Milestone 5) — so a slow backend for one request doesn't block other
  concurrent requests to your server, since each has its own thread (or
  thread-pool worker).
- Match on `TcpStream::connect(...)`'s `Err` and convert it directly into
  your existing `Response` type with status `502`, the same way you already
  handle a missing static file as `404` — same pattern, different status
  code.
- If the backend were something other than Flask (any HTTP server, really),
  none of this changes — the proxy only speaks HTTP, it has no idea what
  language or framework generated the response. That's the general lesson
  here: this is how *any* reverse proxy talks to *any* backend, Flask is
  just a convenient one to run locally for testing.

**Verify it**: run the Flask app on `:5000`, your Rust server on `:7878`,
then `curl http://127.0.0.1:7878/api/hello` and confirm you get Flask's
response back through the Rust server. Kill the Flask process and re-request
to confirm you get a clean `502` instead of a hang or crash. Check
`X-Forwarded-For` arrives correctly on the Flask side (print
`request.headers` there to confirm).

---

## Stretch goals (optional, after the core works)

Pick any of these if you want to keep extending the project — roughly in
order of effort:

- [ ] Query string parsing (`?key=value&...`).
- [ ] Basic logging middleware (method, path, status, duration per request).
- [ ] HTTP/1.1 keep-alive (reuse a connection for multiple requests instead
      of closing after one).
- [ ] Chunked transfer encoding for responses of unknown length.
- [ ] Simple config (port, root directory) via a file or CLI args
      (`std::env::args`, no crate needed for something this small).
- [ ] TLS via `rustls` (this is the point where reaching for a crate is
      appropriate — implementing TLS by hand is its own multi-month project,
      not a learning exercise worth doing from scratch).
- [ ] Graceful shutdown on Ctrl-C (`ctrlc` crate or a signal handler),
      draining in-flight connections before exiting.
- [ ] Streaming reverse proxy: instead of buffering the whole
      request/response in Milestone 8, forward bytes as they arrive on
      either side (useful for large file uploads/downloads through the
      proxy, and required if you ever add chunked encoding support to it).
- [ ] Load-balance across more than one backend instance (round robin over
      a small `Vec` of backend addresses) — the next logical step after a
      single-backend proxy.

---

## Suggested module layout

Not prescriptive — organize however makes sense to you as you go — but a
reasonable shape to grow into once parsing/response/routing all exist:

```
src/
  main.rs          - server bootstrap: bind listener, accept loop
  http/
    request.rs     - Request struct + parser
    response.rs    - Response struct + serialization
    method.rs      - Method enum
    status.rs      - status code -> reason phrase
  router.rs        - route table + dispatch
  static_files.rs  - path resolution + file serving
  threadpool.rs    - (milestone 5 stretch)
  proxy.rs         - outbound connection to backend + request/response relay (milestone 8)
```

## Testing strategy

- Prefer `curl -v` throughout — the `-v` flag shows you the exact
  request/response bytes, which is invaluable for debugging HTTP-level bugs.
- Keep `nc <host> <port>` handy for sending genuinely malformed input that
  `curl` won't construct for you.
- As parsing logic solidifies, add unit tests for the `Request` parser
  directly (feed it byte slices, assert on the parsed struct / error) rather
  than only testing through a live socket — much faster feedback loop.
