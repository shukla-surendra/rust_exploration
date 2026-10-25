// Data types
//
// Every value in Rust has a type, and the compiler must know all of them before
// the program runs (statically typed). Usually it works them out for you:
//   let x = 5;        // inferred: i32 (the default integer)
//   let y = 2.5;      // inferred: f64 (the default float)
//   let z: u8 = 5;    // written out ("annotated")
//
// SCALAR types (one value):
// | Type          | Size        | Range / values                      | Example          |
// |---------------|-------------|-------------------------------------|------------------|
// | i8 i16 i32    | 1 2 4 bytes | signed: negative and positive       | -5, 42           |
// | i64 i128      | 8 16 bytes  | i32: about -2.1 to 2.1 billion      |                  |
// | u8 u16 u32    | 1 2 4 bytes | unsigned: 0 and up only             | 255u8            |
// | u64 u128      | 8 16 bytes  | u8: 0 to 255                        |                  |
// | isize usize   | 8 bytes*    | pointer-sized; usize for indexes    | v.len()          |
// | f32 f64       | 4 8 bytes   | decimals; f64 is default and precise| 3.14             |
// | bool          | 1 byte      | true / false (10_booleans.rs)       | true             |
// | char          | 4 bytes     | ONE Unicode character, single quotes| 'a', 'é', '🦀'   |
//   * on a 64-bit machine
//
// COMPOUND types (several values):
// | Type          | Example                  | Lesson                  |
// |---------------|--------------------------|-------------------------|
// | tuple         | (1, "a", true)           | 24_tuples.rs            |
// | array         | [1, 2, 3]                | 23_arrays_vs_vec.rs     |
//
// Text: &str ("hello", fixed text) and String (growable): 18_strings.rs.
//
// Which integer? Use i32 unless you have a reason: usize for indexes and
// lengths, u8 for bytes, i64/u64 for big counts (file sizes, timestamps).

fn main() {
    // ------------------------------------------------------------------
    // Example 1: Integers and how to write them
    // ------------------------------------------------------------------
    println!("--- Example 1: integers ---");
    let a = 42; // i32 by default
    let b: i64 = -9_000_000_000; // needs i64: too big for i32. _ is just for reading
    let c = 255u8; // type suffix instead of annotation
    let hex = 0xff; // 255 written in hexadecimal
    let oct = 0o17; // 15 in octal
    let bin = 0b1010; // 10 in binary
    let byte = b'A'; // u8 value of the ASCII character 'A' (65)
    println!("a={a} b={b} c={c} hex={hex} oct={oct} bin={bin} byte={byte}");
    // Every integer type knows its limits:
    println!("i8:  {} to {}", i8::MIN, i8::MAX); // -128 to 127
    println!("u8:  {} to {}", u8::MIN, u8::MAX); // 0 to 255
    println!("i32: {} to {}", i32::MIN, i32::MAX);
    println!("u64 max: {}", u64::MAX);

    // ------------------------------------------------------------------
    // Example 2: Integer overflow
    // ------------------------------------------------------------------
    println!("\n--- Example 2: overflow ---");
    // What happens at 255u8 + 1? In a debug build it PANICS (crashes) with
    // "attempt to add with overflow"; in a --release build it wraps to 0.
    // Don't rely on either: pick what you want explicitly.
    let max: u8 = 255;
    println!("checked_add:    {:?}", max.checked_add(1)); // None = "it would overflow"
    println!("wrapping_add:   {}", max.wrapping_add(1)); // 0 (wraps around like a clock)
    println!("saturating_add: {}", max.saturating_add(1)); // 255 (stays at the limit)
    println!("overflowing_add:{:?}", max.overflowing_add(1)); // (0, true)
    let ok: u8 = 200;
    println!("200u8.checked_add(50) = {:?}", ok.checked_add(50)); // Some(250)

    // ------------------------------------------------------------------
    // Example 3: Floats
    // ------------------------------------------------------------------
    println!("\n--- Example 3: floats ---");
    let x = 2.5; // f64
    let y: f32 = 2.5; // f32: less precise, half the memory
    println!("x={x} y={y}");
    // Floats are approximations in binary, so this is NOT exactly 0.3:
    println!("0.1 + 0.2 = {}", 0.1 + 0.2); // 0.30000000000000004
    println!("== 0.3? {}", 0.1 + 0.2 == 0.3); // false! compare with a tolerance:
    println!("close to 0.3? {}", ((0.1 + 0.2) - 0.3_f64).abs() < 1e-10); // true
    println!("1.0 / 0.0 = {}, 0.0 / 0.0 = {}", 1.0 / 0.0, 0.0_f64 / 0.0); // inf, NaN
    println!("sqrt(2) = {:.4}, 2^10 = {}", 2.0_f64.sqrt(), 2.0_f64.powi(10));

    // ------------------------------------------------------------------
    // Example 4: Integer division
    // ------------------------------------------------------------------
    println!("\n--- Example 4: integer vs float division ---");
    println!("7 / 2   = {}", 7 / 2); // 3: integers drop the fraction
    println!("7.0 / 2.0 = {}", 7.0 / 2.0); // 3.5
    println!("7 % 2   = {}", 7 % 2); // 1: the remainder
    // let bad = 7 / 2.0;  // error[E0277]: cannot divide `{integer}` by `{float}`
    //                     // Rust never mixes types silently; convert first (Example 6).

    // ------------------------------------------------------------------
    // Example 5: bool and char
    // ------------------------------------------------------------------
    println!("\n--- Example 5: bool and char ---");
    let is_ready: bool = true;
    let letter = 'R'; // char: single quotes. "R" (double) would be a &str
    let crab = '🦀'; // any Unicode character fits in one char
    println!("is_ready={is_ready} letter={letter} crab={crab}");
    println!("is 'R' uppercase? {}  as lowercase: {}", letter.is_uppercase(), letter.to_ascii_lowercase());
    println!("is '7' a digit? {}  its value: {:?}", '7'.is_ascii_digit(), '7'.to_digit(10)); // Some(7)

    // ------------------------------------------------------------------
    // Example 6: Converting between types with `as`
    // ------------------------------------------------------------------
    println!("\n--- Example 6: casting with as ---");
    let n: i32 = 7;
    let f = n as f64 / 2.0; // int -> float
    println!("7 as f64 / 2.0 = {f}");
    println!("3.99 as i32 = {}", 3.99 as i32); // 3: drops the fraction, no rounding
    println!("3.99 rounded = {}", 3.99_f64.round() as i32); // 4
    let big_num: i32 = 300;
    // (Writing `300 as u8` directly is a compile error: the compiler sees the literal doesn't fit.)
    println!("300 as u8 = {}", big_num as u8); // 44: keeps only the low 8 bits! careful
    println!("-1 as u8 = {}", -1_i32 as u8); // 255
    println!("'A' as u8 = {}, 66 as char = {}", 'A' as u8, 66u8 as char); // 65, B
    println!("true as i32 = {}", true as i32); // 1
    // Safer conversion that refuses values that don't fit:
    println!("u8::try_from(300) = {:?}", u8::try_from(300_i32)); // Err(...)
    println!("u8::try_from(200) = {:?}", u8::try_from(200_i32)); // Ok(200)

    // ------------------------------------------------------------------
    // Example 7: Text to number (parse)
    // ------------------------------------------------------------------
    println!("\n--- Example 7: parse ---");
    // parse can fail ("abc" isn't a number), so it returns a Result (31_results.rs).
    // You tell it the target type with an annotation or ::<type>.
    let good: i32 = "42".parse().unwrap();
    let price = "9.99".parse::<f64>().unwrap();
    println!("parsed {good} and {price}");
    println!("\"abc\".parse::<i32>() = {:?}", "abc".parse::<i32>()); // Err(...)
    println!("\"300\".parse::<u8>()  = {:?}", "300".parse::<u8>()); // Err: too big for u8

    // ------------------------------------------------------------------
    // Example 8: Size in memory
    // ------------------------------------------------------------------
    println!("\n--- Example 8: sizes ---");
    use std::mem::size_of;
    println!("u8={} i32={} i64={} f64={} bool={} char={} usize={}",
        size_of::<u8>(), size_of::<i32>(), size_of::<i64>(), size_of::<f64>(),
        size_of::<bool>(), size_of::<char>(), size_of::<usize>());
}
