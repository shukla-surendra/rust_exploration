# 11. Rust in the AWS Ecosystem

**Python equivalent:** `boto3`. Rust's official AWS SDK
(`aws-sdk-*` crates, generated from the same API models `boto3` is
generated from) covers the same services, with the same general shape
— but it's **async-only**, which `boto3` isn't, so this chapter starts
with the one prerequisite concept none of the other docs in this repo
cover yet: `async`/`await`.

## Prerequisite: `async`/`await`, in brief

Nothing so far in [foundation](../foundation/crates-and-modules.md) or
the [workbook](../workbook/00-how-to-use.md) covers this — [Concurrency](../workbook/08-concurrency.md)
covers OS threads, which is a different tool for a different job.
`async` is for **waiting efficiently** (network calls, disk I/O — lots
of "doing nothing but waiting for a response") rather than **using
multiple CPUs**.

```rust
async fn fetch_thing() -> String {
    // .await suspends this function, yielding the thread to other work,
    // until the underlying I/O actually completes
    some_io_operation().await
}
```

- `async fn` doesn't run when called — it returns a `Future` (a value
  representing "this will produce a `String` eventually"), which does
  nothing until something *drives* it.
- `.await` is where a function actually suspends — while waiting on
  network I/O, the underlying **runtime** (Rust's `std` doesn't include
  one — you add a crate) can run other `.await`-ing tasks on the same
  thread instead of blocking it idle.
- **`tokio`** is the standard runtime almost the entire ecosystem
  (including the AWS SDK) is built on — closest Python analog is
  `asyncio`, and the mental model transfers directly: `async fn` ≈
  Python's `async def`, `.await` ≈ Python's `await`, `tokio::main` ≈
  wrapping your entry point the way `asyncio.run(main())` does.

```rust
#[tokio::main]
async fn main() {
    let result = fetch_thing().await;
    println!("{result}");
}
```

`#[tokio::main]` is a macro that wraps your `async fn main()` in the
boilerplate to actually start the runtime and drive the top-level
future to completion — without it, `async fn main()` alone wouldn't
run at all (nothing would ever `.await` it). This is genuinely enough
async to read and write basic AWS SDK code; for the full picture —
what a `Future` actually is, why the runtime isn't in `std`, running
calls concurrently instead of one `.await` at a time, and the common
pitfalls — see [Async Rust (Deep Dive)](../async-rust/00-is-it-in-the-language-or-not.md),
which ends by making the SQS example below concurrent as its capstone
example.

## The AWS SDK for Rust — same shape as `boto3`, different ceremony

```toml
[dependencies]
aws-config = "1"
aws-sdk-s3 = "1"
tokio = { version = "1", features = ["full"] }
```

```rust
use aws_sdk_s3::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;   // same credential chain as boto3
    let client = Client::new(&config);

    let resp = client
        .list_objects_v2()
        .bucket("my-bucket")
        .send()
        .await?;

    for obj in resp.contents() {
        println!("{}", obj.key().unwrap_or("?"));
    }
    Ok(())
}
```

Compare to the `boto3` version you already know:

```python
import boto3
s3 = boto3.client("s3")
resp = s3.list_objects_v2(Bucket="my-bucket")
for obj in resp["Contents"]:
    print(obj["Key"])
```

Same three steps (build a client, call a method, read the response) —
the Rust differences are exactly the themes from earlier chapters, not
new concepts: every field access on the response is `Option`-wrapped
(`.key()` returns `Option<&str>`, chapter on
[Option, Result & unwrap_or_else](../foundation/option-result.md)), every
call is `async`/`.await`, and the builder pattern (`.bucket(...).send()`)
replaces `boto3`'s keyword arguments because Rust has no equivalent of
Python's `**kwargs`.

## Credentials — the same chain `boto3` uses

`aws_config::load_from_env()` walks the identical resolution order
`boto3`/the AWS CLI use: environment variables
(`AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`) → shared credentials file
(`~/.aws/credentials`) → an EC2/ECS/Lambda instance role, in that order.
Nothing AWS-account-specific to relearn here — the same `aws configure`
setup and the same IAM role attached to compute already works
identically for a Rust program (see
[Configuration & Secrets](./05-configuration-and-secrets.md) for the
general "never hardcode credentials" discipline, unchanged for AWS
specifically).

## Common services, at a glance

| Service | Crate | boto3 equivalent |
|---|---|---|
| S3 | `aws-sdk-s3` | `boto3.client("s3")` |
| DynamoDB | `aws-sdk-dynamodb` | `boto3.client("dynamodb")` / `boto3.resource("dynamodb")` |
| SQS | `aws-sdk-sqs` | `boto3.client("sqs")` |
| Lambda (invoke) | `aws-sdk-lambda` | `boto3.client("lambda")` |
| Secrets Manager | `aws-sdk-secretsmanager` | `boto3.client("secretsmanager")` |
| SNS | `aws-sdk-sns` | `boto3.client("sns")` |

Each is a separate crate (not one giant `aws-sdk` dependency) — add
only the services you actually call, which keeps compile times and
binary size down; `boto3` bundles everything into one package by
comparison.

**Why the ecosystems made opposite choices here, specifically:** in
Python, `import boto3; boto3.client("s3")` / `boto3.client("sqs")` all
come from the *same* installed package — Python doesn't pay a
compile-time cost for unused code sitting in a dependency (an unused
`boto3` submodule just never gets imported/executed), so bundling every
service into one package is essentially free. Rust does pay that cost:
everything in a dependency you add gets compiled, whether you call it or
not. So the ecosystem splits AWS services into separate crates
specifically to keep that cost proportional to what you actually use —
you're not compiling in DynamoDB/Lambda/SNS bindings just because you
needed S3. They all still share one common piece, `aws-config`, which
builds the shared client configuration (region, credentials) that every
per-service `Client::new(&config)` reuses — one `config`, as many
`Client` types built from it as services you're calling:

```toml
[dependencies]
aws-config = "1"
aws-sdk-s3 = "1"
aws-sdk-sqs = "1"
aws-sdk-lambda = "1"
tokio = { version = "1", features = ["full"] }
```

## DynamoDB — the one with the most ceremony difference

```rust
use aws_sdk_dynamodb::types::AttributeValue;

let resp = client
    .get_item()
    .table_name("users")
    .key("id", AttributeValue::S("user-123".to_string()))
    .send()
    .await?;

if let Some(item) = resp.item() {
    if let Some(AttributeValue::S(name)) = item.get("name") {
        println!("name = {name}");
    }
}
```

DynamoDB's schemaless, dynamically-typed attribute values
(`AttributeValue::S`/`::N`/`::Bool`/...) map onto exactly the enum +
`match`/`if let` pattern from
[Structs, Enums & Pattern Matching](../workbook/03-structs-enums-pattern-matching.md) —
where `boto3` hands you a plain Python `dict` with values already
coerced to native types, the Rust SDK hands you an enum you destructure
explicitly, because Rust has no dynamic "could be any type" value the
way a Python dict entry can be.

## SQS — sending, receiving, and deleting messages

```toml
[dependencies]
aws-sdk-sqs = "1"
```

```rust
use aws_sdk_sqs::Client;

let client = Client::new(&config);
let queue_url = "https://sqs.us-east-1.amazonaws.com/123456789012/my-queue";

// send
client
    .send_message()
    .queue_url(queue_url)
    .message_body("hello from rust")
    .send()
    .await?;

// receive — a poll, not a subscription (see below for why)
let resp = client
    .receive_message()
    .queue_url(queue_url)
    .max_number_of_messages(10)
    .wait_time_seconds(20)   // long polling — same idea as boto3's WaitTimeSeconds
    .send()
    .await?;

for msg in resp.messages() {
    println!("{}", msg.body().unwrap_or(""));

    // you must explicitly delete a message after processing it —
    // SQS doesn't know you're "done" with it otherwise
    client
        .delete_message()
        .queue_url(queue_url)
        .receipt_handle(msg.receipt_handle().unwrap())
        .send()
        .await?;
}
```

Compare to `boto3`:

```python
sqs = boto3.client("sqs")
sqs.send_message(QueueUrl=queue_url, MessageBody="hello from python")

resp = sqs.receive_message(QueueUrl=queue_url, MaxNumberOfMessages=10, WaitTimeSeconds=20)
for msg in resp.get("Messages", []):
    print(msg["Body"])
    sqs.delete_message(QueueUrl=queue_url, ReceiptHandle=msg["ReceiptHandle"])
```

Nearly identical shape, method for method — the two Rust-specific things
to notice: `resp.messages()` returns an empty slice rather than `None`
when there's nothing to process (no `.get("Messages", [])` fallback
needed the way `boto3`'s dict access requires), and every field is still
`Option`-wrapped (`msg.body()`, `msg.receipt_handle()`) for the same
reason as S3/DynamoDB above.

**This is polling, not a subscription** — `receive_message` is a
request/response call you loop on yourself (typically inside a `loop {
... }`, see [Syntax & Control Flow](../workbook/01-basics-and-control-flow.md)),
the same as `boto3`. If you want SQS to *push* to your code instead of
polling for it, that's a Lambda with an SQS trigger configured
(AWS-side wiring, not a different Rust API) — the Lambda itself is still
written the same way as the "running Rust as a Lambda function" section
below, just with `Request` being SQS's event shape (`aws_lambda_events::sqs::SqsEvent`)
instead of a custom struct.

## Invoking a Lambda function from Rust (as a client)

Different from *writing* a Lambda (next section) — this is calling an
*existing* Lambda function from other Rust code, the same role as
`boto3.client("lambda").invoke(...)`:

```toml
[dependencies]
aws-sdk-lambda = "1"
serde_json = "1"
```

```rust
use aws_sdk_lambda::{Client, primitives::Blob};

let client = Client::new(&config);

let payload = serde_json::json!({ "name": "ferris" });

let resp = client
    .invoke()
    .function_name("my-function")
    .payload(Blob::new(payload.to_string()))
    .send()
    .await?;

if let Some(blob) = resp.payload() {
    let body: serde_json::Value = serde_json::from_slice(blob.as_ref())?;
    println!("{body}");
}
```

Compare to `boto3`:

```python
lambda_client = boto3.client("lambda")
resp = lambda_client.invoke(
    FunctionName="my-function",
    Payload=json.dumps({"name": "ferris"}),
)
body = json.loads(resp["Payload"].read())
print(body)
```

The one genuinely new piece versus S3/DynamoDB/SQS: Lambda's payload is
raw bytes (`Blob`), not a typed AWS value — because the payload's actual
shape is *your* Lambda's business, not something the SDK can model
ahead of time. That's why `serde_json` shows up here specifically:
`serde_json::json!` builds the request body, `serde_json::from_slice`
parses the response — the same "define what your data looks like with
`#[derive(Deserialize)]` and let `serde` handle the JSON" approach
already used for the Lambda handler's own `Request`/`Response` structs
below, just invoked from the caller's side instead of the callee's.

By default `.invoke()` waits for the function to finish and run
synchronously (`InvocationType::RequestResponse`, the default) — pass
`.invocation_type(InvocationType::Event)` for fire-and-forget, the same
`InvocationType="Event"` option `boto3` exposes.

## AWS Lambda — running Rust as a Lambda function

```toml
[dependencies]
lambda_runtime = "0.13"
tokio = { version = "1", features = ["macros"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request { name: String }

#[derive(Serialize)]
struct Response { message: String }

async fn handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    Ok(Response { message: format!("hello, {}", event.payload.name) })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
```

Structurally the same shape as a Python Lambda handler
(`def handler(event, context):`) — one function, a JSON-shaped input,
a JSON-shaped output (`#[derive(Deserialize)]`/`#[derive(Serialize)]`
via `serde`, the same derive-macro mechanism as `#[derive(Debug)]` from
[Traits](../foundation/traits.md)). The practical draw over a Python
Lambda: no interpreter cold-start, and a compiled binary is typically a
fraction of a Python Lambda's cold-start time — a genuine reason teams
reach for Rust Lambdas specifically for latency-sensitive functions.

**`cargo lambda`** is the deployment tool that packages a Rust binary
into the shape Lambda's custom runtime expects:

```sh
cargo install cargo-lambda
cargo lambda build --release --arm64    # Lambda's Graviton (ARM) runtime
cargo lambda deploy
```

Analogous to `sam build`/`serverless deploy` in the Python Lambda world
— it packages, and can also directly deploy, without you hand-rolling
the zip/layer structure Lambda's custom runtime requires.

## When this is actually worth reaching for

- **You already know boto3 and just need a fast, small binary** calling
  a handful of AWS services — the SDK usage above transfers almost
  directly.
- **Lambda cold-start latency matters** — this is the single most common
  reason teams introduce Rust into an otherwise-Python AWS stack, often
  for exactly one hot-path function rather than a full rewrite.
- **Skip it** if you're prototyping or the workload isn't
  latency/throughput sensitive — `boto3`'s ergonomics (no `.await`, no
  `Option` unwrapping, dynamic typing) are genuinely faster to write
  against for one-off scripts and glue code.
