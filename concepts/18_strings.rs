// Strings: String and &str
//
// Rust has two main string types:
// | Type    | Name          | Owns its text? | Can grow? | Where you get it                     |
// |---------|---------------|----------------|-----------|--------------------------------------|
// | &str    | string slice  | No (borrows)   | No        | "literals", &my_string, &s[0..3]     |
// | String  | owned string  | Yes            | Yes       | String::from, .to_string(), format!  |
//
//   let s = String::from("hello");      let t: &str = &s[1..4];   // "ell"
//
//   stack: s                  heap                      stack: t
//   ┌──────────────┐          ┌───┬───┬───┬───┬───┐     ┌──────────┐
//   │ ptr ─────────┼────────> │ h │ e │ l │ l │ o │ <───┼─ ptr (+1)│
//   │ len = 5      │          └───┴───┴───┴───┴───┘     │ len = 3  │
//   │ capacity = 5 │                                    └──────────┘
//   └──────────────┘
//
// Rule of thumb:
// - Function PARAMETERS: take &str. It accepts both &str and &String.
// - Struct FIELDS and return values you build: use String (it owns the data).
//
// Text is UTF-8: one character can be 1 to 4 BYTES. That's why:
// - s.len() counts bytes, not characters ("é" is 2 bytes).
// - s[0] is NOT allowed (it might land in the middle of a character).
//   Use s.chars() to go character by character.
//
// Cheat sheet:
// | Want to               | Use                                | Gives            |
// |-----------------------|------------------------------------|------------------|
// | append text           | s.push_str("..."), s += "..."      | (changes s)      |
// | append one char       | s.push('!')                        | (changes s)      |
// | join pieces           | format!("{a} {b}")                 | String           |
// | length                | s.len() / s.chars().count()        | bytes / chars    |
// | search                | contains, starts_with, find        | bool / Option    |
// | change case           | to_uppercase, to_lowercase         | new String       |
// | strip spaces          | trim, trim_start, trim_end         | &str             |
// | split                 | split(','), split_whitespace, lines| iterator of &str |
// | replace               | s.replace("a", "b")                | new String       |
// | to a number           | s.parse::<i32>()                   | Result           |

// Takes &str: works with literals AND Strings (pass &my_string).
fn shout(text: &str) -> String {
    format!("{}!", text.to_uppercase())
}

// Returns the first word as a slice of the input: no copying.
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

fn is_palindrome(s: &str) -> bool {
    let clean: Vec<char> = s.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect();
    let reversed: Vec<char> = clean.iter().rev().cloned().collect();
    clean == reversed
}

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Creating strings
    // ------------------------------------------------------------------
    println!("--- Example 1: creating ---");
    let literal: &str = "hello"; // baked into the program, read-only
    let a = String::from("hello"); // String from a literal
    let b = "hello".to_string(); // same thing
    let c = String::new(); // empty String
    let d = format!("{} #{}", "order", 42); // built with formatting
    println!("{literal} | {a} | {b} | '{c}' | {d}");
    // Comparing works across the two types:
    println!("a == literal? {}", a == literal);

    // ------------------------------------------------------------------
    // Example 2: Changing a String
    // ------------------------------------------------------------------
    println!("\n--- Example 2: growing ---");
    let mut s = String::from("Hello"); // must be mut to change
    s.push_str(", World"); // add text
    s.push('!'); // add one char (single quotes)
    s += " Bye"; // += works with a &str
    println!("{s}");
    s.insert(0, '>'); // insert at byte position 0
    s.truncate(14); // keep only the first 14 bytes
    println!("{s}");
    s.clear(); // empty it
    println!("after clear: '{s}' (is_empty = {})", s.is_empty());

    // ------------------------------------------------------------------
    // Example 3: Joining strings
    // ------------------------------------------------------------------
    println!("\n--- Example 3: concatenation ---");
    let first = String::from("Ada");
    let last = String::from("Lovelace");
    // format! borrows everything: first and last stay usable. Usually the best choice.
    let full = format!("{first} {last}");
    println!("{full}");
    // + takes OWNERSHIP of the left String and borrows the right side (&):
    let full2 = first + " " + &last;
    println!("{full2}");
    // println!("{first}");  // error[E0382]: borrow of moved value: `first` (moved by +)
    let words = ["red", "green", "blue"];
    println!("joined: {}", words.join(", "));
    println!("repeated: {}", "ab".repeat(3));

    // ------------------------------------------------------------------
    // Example 4: Length, bytes and characters (UTF-8)
    // ------------------------------------------------------------------
    println!("\n--- Example 4: len vs chars ---");
    let word = "héllo";
    println!("'{word}': len() = {} bytes, chars().count() = {} characters", word.len(), word.chars().count()); // 6 vs 5
    // let h = word[0];  // error[E0277]: the type `str` cannot be indexed by `{integer}`
    println!("first char: {:?}", word.chars().next()); // Some('h')
    println!("third char: {:?}", word.chars().nth(2)); // Some('l')
    for (i, ch) in word.char_indices() {
        print!("{i}:{ch} "); // byte position of each char: note é takes 1 and 2
    }
    println!();
    println!("bytes: {:?}", word.as_bytes());

    // ------------------------------------------------------------------
    // Example 5: Slicing
    // ------------------------------------------------------------------
    println!("\n--- Example 5: slices ---");
    let text = String::from("hello world");
    let hello = &text[0..5]; // bytes 0,1,2,3,4 -> a &str pointing into text
    let world = &text[6..]; // from 6 to the end
    println!("'{hello}' + '{world}'");
    println!("first word: '{}'", first_word(&text));
    // Slicing in the middle of a multi-byte char PANICS at runtime:
    // &"héllo"[0..2]  -> panic: byte index 2 is not a char boundary
    // Safer: .get() returns None instead of panicking.
    println!("\"héllo\".get(0..2) = {:?}", "héllo".get(0..2)); // None
    println!("\"héllo\".get(0..3) = {:?}", "héllo".get(0..3)); // Some("hé")

    // ------------------------------------------------------------------
    // Example 6: Searching and inspecting
    // ------------------------------------------------------------------
    println!("\n--- Example 6: searching ---");
    let line = "ERROR: disk full on /dev/sda1";
    println!("contains \"disk\":   {}", line.contains("disk"));
    println!("starts_with ERROR: {}", line.starts_with("ERROR"));
    println!("ends_with \"1\":     {}", line.ends_with('1'));
    println!("find \"/dev\":       {:?}", line.find("/dev")); // Some(byte index)
    println!("find \"cpu\":        {:?}", line.find("cpu")); // None
    if let Some(rest) = line.strip_prefix("ERROR: ") {
        println!("message part:      {rest}");
    }

    // ------------------------------------------------------------------
    // Example 7: Transforming (each returns a NEW String or &str)
    // ------------------------------------------------------------------
    println!("\n--- Example 7: transforming ---");
    let messy = "   Rust Is Fun   ";
    println!("trim:        '{}'", messy.trim());
    println!("upper:       '{}'", messy.trim().to_uppercase());
    println!("lower:       '{}'", messy.trim().to_lowercase());
    println!("replace:     '{}'", messy.trim().replace("Fun", "Fast"));
    let reversed: String = "stressed".chars().rev().collect();
    println!("reversed:    '{reversed}'"); // desserts
    println!("shout:       '{}'", shout("hey")); // a &str literal
    let owned = String::from("you");
    println!("shout:       '{}'", shout(&owned)); // &String works where &str is expected

    // ------------------------------------------------------------------
    // Example 8: Splitting
    // ------------------------------------------------------------------
    println!("\n--- Example 8: splitting ---");
    let csv = "ann,31,paris";
    let parts: Vec<&str> = csv.split(',').collect();
    println!("split(','): {:?}", parts);
    let (name, rest) = csv.split_once(',').unwrap(); // split at the FIRST comma only
    println!("split_once: name='{name}' rest='{rest}'");
    let sentence = "the  quick   brown fox";
    println!("split_whitespace: {:?}", sentence.split_whitespace().collect::<Vec<_>>());
    let poem = "line one\nline two\nline three";
    for (n, l) in poem.lines().enumerate() {
        println!("  {}: {l}", n + 1);
    }

    // ------------------------------------------------------------------
    // Example 9: Converting
    // ------------------------------------------------------------------
    println!("\n--- Example 9: conversions ---");
    let n: i32 = "42".parse().unwrap(); // &str -> number
    let back = n.to_string(); // number -> String
    let as_str: &str = back.as_str(); // String -> &str (or &back)
    let owned_again: String = as_str.to_owned(); // &str -> String
    println!("{n} | {back} | {as_str} | {owned_again}");
    let chars: Vec<char> = "abc".chars().collect(); // String -> Vec<char>
    let from_chars: String = chars.iter().collect(); // Vec<char> -> String
    println!("{:?} -> {from_chars}", chars);

    // ------------------------------------------------------------------
    // Example 10: Putting it together
    // ------------------------------------------------------------------
    println!("\n--- Example 10: small tasks ---");
    for p in ["Racecar", "A man, a plan, a canal: Panama", "Rust"] {
        println!("'{p}' palindrome? {}", is_palindrome(p));
    }
    let text = "the cat and the hat";
    let count = text.split_whitespace().filter(|w| *w == "the").count();
    println!("'the' appears {count} times");
    let vowels = text.chars().filter(|c| "aeiou".contains(*c)).count();
    println!("{vowels} vowels");
    let title: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect();
    println!("title case: {}", title.join(" "));
}
