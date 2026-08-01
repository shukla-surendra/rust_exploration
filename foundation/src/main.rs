use foundation::welcome::greeting;

fn main() {
    println!("{}", greeting());
    println!("Hello, world!");
}

// 
// cargo new foundation
// 
// #### Single file access in main.rs
// 
// src/welcome.rs
//     --> pub fn greeting() 
// src/main.rs
//     --> mod welcome;  --> define the module for the welcome.rs file
//     this allows us to use the functions defined in welcome.rs similar needs to be done of other files in the project
//     --> use welcome::greeting;
//     --> fn main() {
//             println!("{}", greeting());
//         }
// 
// #### Module Level access in main.rs
// src/gui/mouse.rs
// src/gui/graphics.rs
// src/gui/colors.rs
// src/gui/fonts.rs
// src/gui/widgets.rs
// src/gui/mod.rs
//     -->pub mod mouse;
//     -->pub mod graphics;
//     -->pub mod colors;
//     -->pub mod fonts;
//     -->pub mod widgets;
// src/main.rs
//     -->mod gui;
//     -->use gui::mouse;
//     -->use gui::graphics;
//     -->use gui::colors;
//     -->use gui::fonts;
//     -->use gui::widgets;
// 
// 
// 
// Package vs. crate vs. module — three different words
// 
//   - Package: what cargo new foundation creates. Defined by Cargo.toml. One package can contain multiple crates.
//   - Crate: the actual unit the compiler compiles. Each crate has one root file and one module tree.
//     - A binary crate has a fn main() and compiles to an executable. src/main.rs is one; each file in src/bin/*.rs is another. A package can have
//   many binary crates.
//     - A library crate has no main(), compiles to an .rlib, and is meant to be depended on. A package can have at most one, rooted at src/lib.rs,
//   named after [package] name.
//   - Module: how you split code into namespaced pieces (mod foo;, foo/mod.rs, etc.) inside a crate. Your two examples — mod welcome; and
//   gui/mod.rs — are both module trees living inside the main.rs binary crate. They never leave that crate.
// 
// a package can have  
// 
// - Library crates: 0 or 1
// - Binary crates: 0 or more (any number)
// - Binary crate: has a fn main(), compiles to an executable.
// - Library crate: no fn main(), compiles to code meant to be used by other crates.
// 
// ┌───────────┬─────────────────────────────────────────────────────────────────────────────┬──────────────────────────────────────────────┐
// │           │                                Library crate                                │                 Binary crate                 │
// ├───────────┼─────────────────────────────────────────────────────────────────────────────┼──────────────────────────────────────────────┤
// │ How       │ Add src/lib.rs                                                              │ Add src/main.rs, or any .rs file under       │
// │ created   │                                                                             │ src/bin/                                     │
// ├───────────┼─────────────────────────────────────────────────────────────────────────────┼──────────────────────────────────────────────┤
// │ Where     │ src/lib.rs (only this one path)                                             │ src/main.rs, or src/bin/*.rs (one file = one │
// │           │                                                                             │  binary crate)                               │
// ├───────────┼─────────────────────────────────────────────────────────────────────────────┼──────────────────────────────────────────────┤
// │ Entry     │ none (no fn main()) — the crate root file itself, whose top-level items are │ fn main() inside that file                   │
// │ point     │  the "entry" other crates see                                               │                                              │
// ├───────────┼─────────────────────────────────────────────────────────────────────────────┼──────────────────────────────────────────────┤
// │ Purpose   │ Hold shared code/logic to be reused by other crates                         │ Produce a runnable executable                │
// └───────────┴─────────────────────────────────────────────────────────────────────────────┴──────────────────────────────────────────────┘
//
// #### Library crate access in main.rs
//
// src/welcome.rs
//     --> pub fn greeting()
// src/lib.rs
//     --> pub mod welcome;  --> re-export the module for other crates/binaries to use
// src/main.rs
//     --> use foundation::welcome::greeting;
//     --> fn main() {
//             println!("{}", greeting());
//         }
//
// no `mod lib;` anywhere — src/lib.rs is auto-recognized by Cargo as the library
// crate root (fixed path), named after [package] name in Cargo.toml. That's why
// main.rs reaches it as `foundation::...` with no explicit import wiring.
//
// why `foundation::` and not just `welcome::` like the mod example above?
// `mod welcome;` puts welcome.rs inside main.rs's own crate (no prefix needed).
// `pub mod welcome;` in lib.rs puts it inside a *different* crate (the library),
// so main.rs must cross a crate boundary — paths into another crate start with
// that crate's name. alternatives to the name "foundation":
//   - rename the lib crate: Cargo.toml [lib] name = "found_lib"  --> use found_lib::welcome::...;
//   - alias on import:      use foundation as f;                --> f::welcome::...;
//   - or just import the item directly, no prefix at call site: use foundation::welcome::greeting; --> greeting()
//
// #### multiple binary crates in one package
// a package: 0-1 library crate, but any number of binary crates (each own fn main()).
// useful for sharing one lib.rs across several runnable programs (cli, server, etc).
// src/main.rs        --> default binary, named after the package ("foundation")
// src/bin/server.rs  --> extra binary crate, named "server"
// src/bin/client.rs  --> extra binary crate, named "client"
// each src/bin/*.rs file can `use foundation::...;` same as main.rs does.
// run a specific one: cargo run --bin server

