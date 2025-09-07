struct Deck{
    cards: Vec<String>,
}

fn main(){
    let mut deck = Deck{ cards: vec![]};
    deck.cards.push("Hello".to_string());
    println!("{:?}", deck.cards)
}