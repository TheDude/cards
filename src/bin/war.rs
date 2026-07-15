use std::io::{ self, BufRead, BufReader, Stdin, Stdout, Write};

use deck::{self, Deck};
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

    fn write_flush(&mut self, msg: &str) -> std::io::Result<()>{
        writeln!(self.writer, "{}", msg)?; //early return std::io::Result<()>
        self.writer.flush()     //returns std::io::Result<()>
    }

    fn deal(&mut self){
        let mut i = 0;
        let player_count = self.dealer.cards.len();
        while let Some(card) = self.dealer.cards.pop(){
            self.players[(i + 1) % player_count].cards.push(card);
            i += 1;
        }
    }
}


//game initialization
    //create 2 default decks for player hands
    //create 52 card deck for deal
    //deal() cards from deck, consuming the main deck.
    
    //game loop
    //check if either player deck is empty, if either is empty, they are the loser and run game_over()
    //deal() 1 card from player 1, then player 2. Always draw from top.
    //display cards, deck count for each player, wait for enter key. 
    //compare cards:
    //if cards are equal do a war()
    //else, determine who had the higher card and add both cards to the bottom of their deck()

    //war:
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
            Err(e) => { println!("Invalid Value: {}", e) } //try again
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

   
}