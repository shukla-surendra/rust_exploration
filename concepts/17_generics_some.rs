// Using a nested generic type: Option<Vec<i32>>
//
// Read it inside out (see 16_generics.rs):
//   i32               an integer
//   Vec<i32>          a list of integers
//   Option<Vec<i32>>  maybe a list of integers, maybe nothing
//
// So get_data() returns one of:
//   Some(vec![10, 20, 30])  -> here's the list
//   None                    -> no list at all (different from an empty list!)
//
// Try: change get_data() to return None and run again.

fn get_data() -> Option<Vec<i32>> {
    // vec![...] is a macro that builds a Vec. Some(...) wraps it in the Option.
    Some(vec![10, 20, 30])
}

fn main() {
    let data = get_data();

    // match opens the Option: one arm per possibility.
    match data {
        // Some(vec): the list is there; `vec` is a new variable holding it.
        Some(vec) => {
            // `for num in vec` takes the Vec by value (moves it) and gives each
            // number in order. Use `for num in &vec` to only borrow it.
            for num in vec {
                println!("{}", num);
            }
        }
        None => println!("No data available."),
    }
}
