use std::fmt;
use rand::RngExt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CardValue {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

impl CardValue {
    pub fn all() -> [CardValue; 13] {
        [
            CardValue::Ace,
            CardValue::King,
            CardValue::Queen,
            CardValue::Jack,
            CardValue::Ten,
            CardValue::Nine,
            CardValue::Eight,
            CardValue::Seven,
            CardValue::Six,
            CardValue::Five,
            CardValue::Four,
            CardValue::Three,
            CardValue::Two,
        ]
    }
}

impl fmt::Display for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            CardValue::King => "K",
            CardValue::Queen => "Q",
            CardValue::Jack => "J",
            CardValue::Ten => "10",
            CardValue::Nine => "9",
            CardValue::Eight => "8",
            CardValue::Seven => "7",
            CardValue::Six => "6",
            CardValue::Five => "5",
            CardValue::Four => "4",
            CardValue::Three => "3",
            CardValue::Two => "2",
            CardValue::Ace => "A"
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub fn all() -> [Suit; 4] {
        [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Suit::Spades => "♠️",
            Suit::Hearts => "♥️",
            Suit::Diamonds => "♦️",
            Suit::Clubs => "♣️"
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Card {
    suit: Suit,
    value: CardValue,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.suit, self.value)
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Deck {
            cards: Suit::all()
                .into_iter()
                .flat_map(|suit| {
                    CardValue::all().into_iter().map(move |face_value| Card {
                        value: face_value,
                        suit: suit,
                    })
                })
                .collect(),
        }
    }

    pub fn shuffle(&mut self){
        let mut rng = rand::rng();
        Deck::shuffle_rng(self, &mut rng);
    }

    fn shuffle_rng<R: rand::Rng>(&mut self, rng: &mut R) {
        //get a random number generator 
        //iterate over cards in reverse, starting at the end
        for index in 0..self.cards.len(){
            //pick a random index in the range [0..card]
            let random_index = rng.random_range(0..=index);
            //swap those 2 indicies
            self.cards.swap(index, random_index);
        }
    }

    fn draw_n(&mut self, count: usize) -> Result<Vec<Card>, String>{
        if count > self.cards.len(){
            return Err(format!("Count({}) is larger than deck({})", count, self.cards.len()))
        }
        let deal = self.cards.drain(0..count).collect();
        Ok(deal)
    }

    fn draw(&mut self) -> Result<Card, String>{
        if self.cards.len() < 1 {
            return Err(format!("Empty Deck"))
        }
        Ok(self.cards.remove(0))
    }

    fn push_bottom(&mut self, card: Card){
        self.cards.push(card);
    }

    fn push_top(&mut self, card: Card){
        self.cards.insert(0, card);
    }

}

impl fmt::Display for Deck{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        self.cards.iter().map( |card| {
            card.fmt(f)
        }).collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::{SeedableRng};

    #[test]
    fn test_shuffle() {
        let deck = Deck::new();
        let mut deck2 = deck.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        Deck::shuffle_rng(&mut deck2, &mut rng);
        assert_eq!(deck.cards.len(), deck2.cards.len());
        let diff = deck.cards.iter().zip(deck2.cards.iter()).any(|(card1, card2)| *card1 != *card2);
        assert!(diff, "Shuffled deck is in exact same order as before!");
    }

    #[test]
    fn test_draw_n(){
        let mut deck = Deck::new();
        let drawn = vec![
            Card{value: CardValue::Ace, suit: Suit::Spades},
            Card{value: CardValue::King, suit: Suit::Spades},
            Card{value: CardValue::Queen, suit: Suit::Spades},
            Card{value: CardValue::Jack, suit: Suit::Spades} 
        ];
        assert_eq!(deck.draw_n(4).unwrap(), drawn);
    }

    #[test]
    fn test_draw(){
        let mut deck = Deck::new();
        let drawn = Card{value: CardValue::Ace, suit: Suit::Spades};
        assert_eq!(deck.draw().unwrap(), drawn);
    }

    #[test]
    fn test_default() {
        let deck = Deck::default();
        assert_eq!(deck.cards.len(), 0)
    }

    #[test]
    fn test_push_bottom() {
        let mut deck = Deck::default();
        let card = Card{value: CardValue::Ace, suit: Suit::Spades};
        let card2 = card.clone();
        deck.push_bottom(card2);
        assert_eq!(deck.cards.len(), 1);
        assert_eq!(deck.cards[0], card);
    }

    #[test]
    fn test_push_top(){
        let mut deck = Deck::new();
        let card = Card {value: CardValue::Ace, suit: Suit::Hearts};
        let card2 = card.clone();
        deck.push_top(card);
        assert_eq!(deck.cards.len(), 53);
        assert_eq!(deck.cards[0], card2);
    }
}
