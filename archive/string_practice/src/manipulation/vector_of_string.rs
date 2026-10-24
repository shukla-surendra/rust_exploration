pub fn main_test(){
    let mut games: Vec<String> = vec![];
    games.push("Hello".to_string());
    games.push("Hello".to_string());
    games.push(String::from("Welcome"));
    games.push(String::from("New String"));
    games.push(String::from("End String"));
    println!("{:?}", games);
}