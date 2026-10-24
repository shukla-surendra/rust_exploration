# Crates & Modules

> **Coming from Python/Java:** in Python, every `.py` file is
> automatically a module, and a folder is automatically a package — the
> import system discovers files on disk for you. In Java, a file *is* a
> class, and the folder structure must physically mirror the `package`
> statement. Rust does **neither** — nothing is wired in automatically.
> A file sitting in `src/` isn't part of your program until some `mod`
> declaration says so. That's the one habit to unlearn here: adding a
> `.rs` file is not enough by itself, the same way `import foo` in
> Python "just works" the moment `foo.py` exists on the path.
>
> **Reach for multiple binary crates** (last section below) whenever you
> have several related executables that should share core logic — think
> a CLI and a server for the same product — instead of copy-pasting
> shared code between separate Python scripts or separate Java `main`
> classes.

This page uses a scratch package created with `cargo new foundation` to
demonstrate the wiring. The setup below (`src/welcome.rs` +
`src/main.rs`) is intentionally minimal — the point is the `mod`/`use`
plumbing, not what `greeting()` actually does.

## Single-file access in main.rs

```
src/welcome.rs
    --> pub fn greeting()
src/main.rs
    --> mod welcome;              // define the module for welcome.rs
    --> use welcome::greeting;
    --> fn main() {
            println!("{}", greeting());
        }
```

`mod welcome;` puts `welcome.rs` inside `main.rs`'s own crate, so it's
reachable without any crate-name prefix. The same pattern applies to any
other file added to the project.

## Module-level access in main.rs

```
src/gui/mouse.rs
src/gui/graphics.rs
src/gui/colors.rs
src/gui/fonts.rs
src/gui/widgets.rs
src/gui/mod.rs
    --> pub mod mouse;
    --> pub mod graphics;
    --> pub mod colors;
    --> pub mod fonts;
    --> pub mod widgets;
src/main.rs
    --> mod gui;
    --> use gui::mouse;
    --> use gui::graphics;
    --> use gui::colors;
    --> use gui::fonts;
    --> use gui::widgets;
```

## Package vs. crate vs. module

- **Package** — what `cargo new foundation` creates, defined by `Cargo.toml`. One package can contain multiple crates.
- **Crate** — the actual unit the compiler compiles. Each crate has one root file and one module tree.
  - A **binary crate** has a `fn main()` and compiles to an executable. `src/main.rs` is one; each file in `src/bin/*.rs` is another. A package can have many binary crates.
  - A **library crate** has no `main()`, compiles to an `.rlib`, and is meant to be depended on. A package can have at most one, rooted at `src/lib.rs`, named after `[package] name`.
- **Module** — how you split code into namespaced pieces (`mod foo;`, `foo/mod.rs`, etc.) inside a crate. `mod welcome;` and `gui/mod.rs` are both module trees living inside the `main.rs` binary crate — they never leave that crate.

A package can have:

- Library crates: 0 or 1
- Binary crates: 0 or more (any number)

| | Library crate | Binary crate |
|---|---|---|
| **How created** | Add `src/lib.rs` | Add `src/main.rs`, or any `.rs` file under `src/bin/` |
| **Where** | `src/lib.rs` (only this one path) | `src/main.rs`, or `src/bin/*.rs` (one file = one binary crate) |
| **Entry point** | none (no `fn main()`) — the crate root file itself, whose top-level items are the "entry" other crates see | `fn main()` inside that file |
| **Purpose** | Hold shared code/logic to be reused by other crates | Produce a runnable executable |

## Library crate access in main.rs

```
src/welcome.rs
    --> pub fn greeting()
src/lib.rs
    --> pub mod welcome;          // re-export the module for other crates/binaries to use
src/main.rs
    --> use foundation::welcome::greeting;
    --> fn main() {
            println!("{}", greeting());
        }
```

No `mod lib;` anywhere — `src/lib.rs` is auto-recognized by Cargo as the
library crate root (fixed path), named after `[package] name` in
`Cargo.toml`. That's why `main.rs` reaches it as `foundation::...` with no
explicit import wiring.

### Why `foundation::` and not just `welcome::` like the mod example above?

`mod welcome;` puts `welcome.rs` inside `main.rs`'s own crate — no prefix
needed. `pub mod welcome;` in `lib.rs` puts it inside a *different* crate
(the library), so `main.rs` must cross a crate boundary — paths into
another crate start with that crate's name.

Alternatives to the name `foundation`:

- **Rename the lib crate** — `Cargo.toml`: `[lib] name = "found_lib"` → `use found_lib::welcome::...;`
- **Alias on import** — `use foundation as f;` → `f::welcome::...;`
- **Import the item directly**, no prefix at the call site — `use foundation::welcome::greeting;` → `greeting()`

## Multiple binary crates in one package

A package: 0–1 library crate, but any number of binary crates (each with
its own `fn main()`). Useful for sharing one `lib.rs` across several
runnable programs (cli, server, etc).

```
src/main.rs        --> default binary, named after the package ("foundation")
src/bin/server.rs  --> extra binary crate, named "server"
src/bin/client.rs  --> extra binary crate, named "client"
```

Each `src/bin/*.rs` file can `use foundation::...;` the same way `main.rs`
does. Run a specific one with:

```
cargo run --bin server
```
