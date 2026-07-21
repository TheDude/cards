use std::io::{ self, BufRead, BufReader, Stdin, Stdout, Write};

use deck::{self, Deck, Card, CardValue, Suit};
use rand;

struct Table<N: rand::RngExt = rand::rngs::StdRng, R: BufRead = BufReader<Stdin>, W: Write = Stdout> {
    //testing injectables
    rng: N,
    reader: R,
    writer: W,

    //actual data
    dealer: Deck,
    players: Vec<Deck>,
}

impl Table<rand::rngs::StdRng, BufReader<Stdin>, Stdout> {
    fn new () -> Self{
        let table = Table { 
            rng: rand::make_rng(),
            reader: BufReader::new(io::stdin()),
            writer: io::stdout(),
            dealer: Deck::new(),
            players: Vec::new(),
        };
        table
    }
}

impl <N: rand::RngExt, R: BufRead, W: Write> Table<N, R, W> {
    fn new_unit_test(rng: N, reader: R, writer: W) -> Self{
        Table{
            rng: rng,
            reader: reader,
            writer: writer,
            dealer: Deck::new(),
            players: Vec::new(),
        }
    }

    fn set_player_count(&mut self, player_count: usize){
        for _i in 0..player_count{
            self.players.push(Deck::default())
        };
    }

    fn initialize(&mut self) -> Result<usize, std::num::ParseIntError>{
        while let Err(e) =  self.write_flush("Lets Play WAR!"){
            println!("Problem with output: {}", e);
        }

        let mut input = String::new();
        while let Err(e) = self.reader.read_line(&mut input){
            println!("Problem with input: {}", e);
        }
        
        let player_count = input.trim().parse()?;
        Ok(player_count)
    }

    fn game_loop(&mut self){
        //check for empty decks. If only one player has all the cards, he is the winner.
        //deal 1 card from all players. Highest card wins all cards which are added to the bottom of the deck
        //display cards counts, cards in battle, and waith for enter key
        //highest card is winner, if there is more than one highest card, do a WAR.
        //

        while let None = self.check_win() {
            //put all the battle cards into a working vec.
            let battle_cards = self.play_cards();
            let winners = self.find_winners(battle_cards);
        }
    }

    fn play_cards(&mut self) -> Vec<(usize, Card)>{
        let mut battle_cards: Vec<(usize, Card)> = Vec::new();
        for (i, player) in self.players.iter_mut().enumerate() {
            if let Some(card) = player.cards.pop(){
                battle_cards.push((i, card));
            }
        }
        battle_cards
    }

    fn find_winners(&mut self, battle_cards: Vec<(usize, Card)>) -> Vec<usize>{
        //find the winning card indicies
        let mut winners: Vec<usize> = Vec::new();
        winners = battle_cards.iter().enumerate().fold(winners, |mut acc, x| {
            let index = x.0;
            match acc.first(){
                None => {
                    acc.push(index);
                    acc
                },
                Some(max) if battle_cards[*max].1.value < battle_cards[index].1.value => {
                    acc.clear();
                    acc.push(index);
                    acc
                },
                Some(max) if battle_cards[*max].1.value == battle_cards[index].1.value => {
                    acc.push(index);
                    acc
                }
                _ => {
                    acc
                }
            }
        });
        winners
    }

    fn check_win(&self) -> Option<usize>{
        let non_empty: Vec<usize> = self.players.iter().enumerate().filter(|(_, p)| !p.cards.is_empty()).map(|(idx, _)| idx).collect();

        match non_empty.as_slice() {
            [idx] => Some(*idx),
            _ => None,
        }
    }

    fn write_flush(&mut self, msg: &str) -> std::io::Result<()>{
        writeln!(self.writer, "{}", msg)?; //early return std::io::Result<()>
        self.writer.flush()     //returns std::io::Result<()>
    }

    fn deal(&mut self){
        let mut i = 0;
        let player_count = self.players.len();
        while let Some(card) = self.dealer.cards.pop(){
            self.players[i % player_count].cards.push(card);
            i += 1;
        }
    }
}


//game initialization
    //INIT:
    //create 2 default decks for player hands
    //create 52 card deck for deal
    //deal() cards from deck, consuming the main deck.
    
    //GAME LOOP:
    //check if either player deck is empty, if either is empty, they are the loser and run game_over()
    //deal() 1 card from player 1, then player 2. Always draw from top.
    //display cards, deck count for each player, wait for enter key. 
    //compare cards:
    //if cards are equal do a war()
    //else, determine who had the higher card and add both cards to the bottom of their deck()

    //WAR: 
    //deal 4 cards from each player deck. 
    //display 3 placeholder cards + 4th card from each player, wait for enter key
    //display all 4 cards from each player
    //compare 4th cards:
    //like in normal play, the player with the highest rank card for the 4th card takes all cards and adds them
    //to the bottom of their deck. 

    //game_over:
    //display "Player X wins! and show card count." wait for enter key.
    //drop all decks and run game_init() again.

fn main(){
    let mut table = Table::new();

    let player_count: usize = loop {
        match table.initialize() {
            Ok(val) => { break val }
            Err(e) => { println!("Invalid value: CardValue::{}", e) } //try again
        };
    };

    table.set_player_count(player_count);
    table.dealer.shuffle(&mut table.rng);
    table.deal();
}

#[cfg(test)]
mod tests {

    use rand::SeedableRng;

use super::*;

    #[test]
    fn test_table_new(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let table = Table::new_unit_test(rng, reader, writer);

        assert_eq!(table.dealer.cards.len(), 52);
        assert_eq!(table.players.len(), 0);

    }

    #[test]
    fn test_set_player_count(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"3\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);

        assert_eq!(table.dealer.cards.len(), 52);
        assert_eq!(table.dealer.cards, Deck::new().cards);
        assert_eq!(table.players.len(), 3);
        assert_eq!(table.players[0].cards.len(), 0);
    }

    #[test]
    fn test_initialize() {

        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"3\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);
        assert_eq!(table.initialize().unwrap(), 3);
        assert_eq!(String::from_utf8(table.writer).unwrap(), "Lets Play WAR!\n")
    }

    #[test]
    fn test_deal(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);
        table.deal();

        assert_eq!(table.players[0].cards.len(), 18);
        assert_eq!(table.players[1].cards.len(), 17);
        assert_eq!(table.players[2].cards.len(), 17);
        assert_eq!(table.dealer.cards.len(), 0);

        let correct_players = vec![Deck{cards: vec![Card { suit: Suit::Clubs, value: CardValue::Two }, Card { suit: Suit::Clubs, value: CardValue::Five }, Card { suit: Suit::Clubs, value: CardValue::Eight }, Card { suit: Suit::Clubs, value: CardValue::Jack }, Card { suit: Suit::Clubs, value: CardValue::Ace }, Card { suit: Suit::Diamonds, value: CardValue::Four }, Card { suit: Suit::Diamonds, value: CardValue::Seven }, Card { suit: Suit::Diamonds, value: CardValue::Ten }, Card { suit: Suit::Diamonds, value: CardValue::King }, Card { suit: Suit::Hearts, value: CardValue::Three }, Card { suit: Suit::Hearts, value: CardValue::Six }, Card { suit: Suit::Hearts, value: CardValue::Nine }, Card { suit: Suit::Hearts, value: CardValue::Queen }, Card { suit: Suit::Spades, value: CardValue::Two }, Card { suit: Suit::Spades, value: CardValue::Five }, Card { suit: Suit::Spades, value: CardValue::Eight }, Card { suit: Suit::Spades, value: CardValue::Jack }, Card { suit: Suit::Spades, value: CardValue::Ace }] }, 
        Deck { cards: vec![Card { suit: Suit::Clubs, value: CardValue::Three }, Card { suit: Suit::Clubs, value: CardValue::Six }, Card { suit: Suit::Clubs, value: CardValue::Nine }, Card { suit: Suit::Clubs, value: CardValue::Queen }, Card { suit: Suit::Diamonds, value: CardValue::Two }, Card { suit: Suit::Diamonds, value: CardValue::Five }, Card { suit: Suit::Diamonds, value: CardValue::Eight }, Card { suit: Suit::Diamonds, value: CardValue::Jack }, Card { suit: Suit::Diamonds, value: CardValue::Ace }, Card { suit: Suit::Hearts, value: CardValue::Four }, Card { suit: Suit::Hearts, value: CardValue::Seven }, Card{ suit: Suit::Hearts, value: CardValue::Ten }, Card { suit: Suit::Hearts, value: CardValue::King }, Card { suit: Suit::Spades, value: CardValue::Three }, Card { suit: Suit::Spades, value: CardValue::Six }, Card { suit: Suit::Spades, value: CardValue::Nine }, Card { suit: Suit::Spades, value: CardValue::Queen }] },
        Deck { cards: vec![Card { suit: Suit::Clubs, value: CardValue::Four }, Card { suit: Suit::Clubs, value: CardValue::Seven }, Card { suit: Suit::Clubs, value: CardValue::Ten }, Card { suit: Suit::Clubs, value: CardValue::King }, Card { suit: Suit::Diamonds, value: CardValue::Three }, Card { suit: Suit::Diamonds, value: CardValue::Six }, Card { suit: Suit::Diamonds, value: CardValue::Nine }, Card { suit: Suit::Diamonds, value: CardValue::Queen }, Card { suit: Suit::Hearts, value: CardValue::Two }, Card { suit: Suit::Hearts, value: CardValue::Five }, Card { suit: Suit::Hearts, value: CardValue::Eight }, Card {suit: Suit::Hearts, value: CardValue::Jack }, Card { suit: Suit::Hearts, value: CardValue::Ace }, Card { suit: Suit::Spades, value: CardValue::Four }, Card { suit: Suit::Spades, value: CardValue::Seven }, Card {suit: Suit::Spades, value: CardValue::Ten }, Card { suit: Suit::Spades, value: CardValue::King }] }
        ];

        assert_eq!(table.players, correct_players);
        
    }

    #[test]
    fn test_check_win() {
        
    }

    #[test]
    fn test_play_cards() {
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);
        table.deal();

        let cards = table.play_cards();
        let correct_cards = vec![
            (0, Card{suit: Suit::Spades, value: CardValue::Ace}),
            (1, Card{suit: Suit::Spades, value: CardValue::Queen}),
            (2, Card{suit: Suit::Spades, value: CardValue::King}),
        ];
        assert_eq!(cards, correct_cards);
    }

    #[test]
    fn test_find_winners() {
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);
        table.deal();

        let cards = table.play_cards();
        let winners = table.find_winners(cards);

        let correct_winners = vec![0];
    
        assert_eq!(winners, correct_winners);

        for player in table.players.iter_mut(){
            player.shuffle(&mut table.rng);
        }
        let cards = table.play_cards();
        let winners = table.find_winners(cards);

        let correct_winners = vec![2];
        assert_eq!(winners, correct_winners);

        for player in table.players.iter_mut(){
            player.shuffle(&mut table.rng);
        }
        let cards = table.play_cards();
        let winners = table.find_winners(cards);
        assert_eq!(winners, vec![1]);

        table.players[0].cards.push(Card{suit: Suit::Spades, value: CardValue::King});
        let cards = table.play_cards();
        assert_eq!(table.find_winners(cards), vec![0,1]);

        table.players[0].cards.push(Card{suit: Suit::Clubs, value: CardValue::Ace});
        table.players[1].cards.push(Card{suit: Suit::Hearts, value: CardValue::Three});
        table.players[2].cards.push(Card{suit: Suit::Clubs, value: CardValue::Three});

        let cards = table.play_cards();
        assert_eq!(table.find_winners(cards), vec![0]);
    }
   
}