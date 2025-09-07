#[derive(Debug)]
struct Deck{
    cards: Vec<String>,
}


fn main(){
    let suits = ["Diamonds", "Club", "Gold"];
    let values = ["Ace", "Two", "Three"];
    let mut cards = vec![];
    for suit in suits{
        for value in values{
            let card = format!("{} of {}", value, suit);
            cards.push(card);
        }
    }
    let mut deck = Deck{ cards }; // short hand for cards : cards if identical names
   
    
    println!("{:?}", deck)
}