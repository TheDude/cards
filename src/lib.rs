use std::fmt;
use rand::RngExt;

#[derive(Clone, Copy, PartialEq)]
pub enum CardValue {
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
    Ace,
}

impl CardValue {
    pub fn all() -> [CardValue; 13] {
        [
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
            CardValue::Ace,
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

#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub struct Card {
    value: CardValue,
    suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.suit, self.value)
    }
}

#[derive(Default, Clone, PartialEq)]
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

    pub fn shuffle(deck: &mut Deck){
        let mut rng = rand::rng();
        Deck::shuffle_rng(deck, &mut rng);
    }

    fn shuffle_rng<R: rand::Rng>(deck: &mut Deck, rng: &mut R) {
        //get a random number generator 
        //iterate over cards in reverse, starting at the end
        for index in 0..deck.cards.len(){
            //pick a random index in the range [0..card]
            let random_index = rng.random_range(0..=index);
            //swap those 2 indicies
            deck.cards.swap(index, random_index);
        }
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
        let mut deck = Deck::new();
        print!("New deck created: {}\n", deck);
        let mut deck2 = deck.clone();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        Deck::shuffle_rng(&mut deck2, &mut rng);
        print!("Shuffled deck: {}\n", deck2);
        assert_eq!(deck.cards.len(), deck2.cards.len());
        let diff = deck.cards.iter().zip(deck2.cards.iter()).any(|(card1, card2)| *card1 != *card2);
        assert!(diff, "Shuffled deck is in exact same order as before!");
        Deck::shuffle(&mut deck);
        print!("Shuffled deck again: {}\n", deck);
    }

    #[test]
    fn test_default() {
        let deck = Deck::default();
        print!("New Empty Deck: {}\n", deck);
        assert!(true)
    }
}
