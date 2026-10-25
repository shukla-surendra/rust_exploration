fn get_data() -> Option<Vec<i32>> {
    Some(vec![10, 20, 30])
}

fn main() {
    let data = get_data();

    match data {
        Some(vec) => {
            for num in vec {
                println!("{}", num);
            }
        }
        None => println!("No data available."),
    }
}