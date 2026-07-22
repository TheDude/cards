use std::io::{self, BufRead, BufReader, Stdin, Stdout, Write};

use deck::{self, Card, Suit, CardValue, Deck};
use rand;

struct Table<N: rand::RngExt = rand::rngs::StdRng, R: BufRead = BufReader<Stdin>, W: Write = Stdout>
{
    //testing injectables
    rng: N,
    reader: R,
    writer: W,

    //actual data
    dealer: Deck,
    players: Vec<Deck>,
    spoils_of_war: Vec<Card>,
    battle_cards: Vec<(usize, Card)>,
    winners: Vec<usize>,
}

impl Table<rand::rngs::StdRng, BufReader<Stdin>, Stdout> {
    fn new() -> Self {
        let table = Table {
            rng: rand::make_rng(),
            reader: BufReader::new(io::stdin()),
            writer: io::stdout(),
            dealer: Deck::new(),
            players: Vec::new(),
            battle_cards: Vec::new(),
            spoils_of_war: Vec::new(),
            winners: Vec::new(),
        };
        table
    }
}

impl<N: rand::RngExt, R: BufRead, W: Write> Table<N, R, W> {
    fn new_unit_test(rng: N, reader: R, writer: W) -> Self {
        Table {
            rng: rng,
            reader: reader,
            writer: writer,
            dealer: Deck::new(),
            players: Vec::new(),
            battle_cards: Vec::new(),
            spoils_of_war: Vec::new(),
            winners: Vec::new(),
        }
    }

    fn set_player_count(&mut self, player_count: usize) {
        for _i in 0..player_count {
            self.players.push(Deck::default())
        }
    }

    fn initialize(&mut self) -> Result<usize, std::num::ParseIntError> {
        while let Err(e) = self.write_flush("Lets Play WAR!") {
            println!("Problem with output: {}", e);
        }

        let mut input = String::new();
        while let Err(e) = self.reader.read_line(&mut input) {
            println!("Problem with input: {}", e);
        }

        let player_count = input.trim().parse()?;
        Ok(player_count)
    }

    fn game_loop(&mut self) {
        //check for empty decks. If only one player has all the cards, he is the winner.
        //deal 1 card from all players. Highest card wins all cards which are added to the bottom of the deck
        //display cards counts, cards in battle, and waith for enter key
        //highest card is winner, if there is more than one highest card, do a WAR.

        while let None = self.check_win() {
            loop{
                self.play_cards();
                self.find_winners();
                match self.winners.len(){
                    1 => {
                        self.award_player();
                        break;
                    }
                    n if n > 1 => { //WAR
                        //There are 2 unique cases for a participant that has less than the required 4 for a War.  
                        //Special Case 1: The player has no additional cards left. This means his battle_card will be the deciding card 
                        //and he fronts no spoils cards. Check for this condition before moving the battle_cards to the spoils
                        //pile.
                        self.ensure_player_has_last_card();

                        //collect 3 cards from each player as the spoils of war
                        self.build_spoils_of_war();
                        
                        //clear the winners vec for the next face-off
                        self.winners.clear();
                    }
                    _ => unreachable!()
                }
            }
        }
    }

    fn build_spoils_of_war(&mut self){
        //append battle cards to spoils stack
        self.spoils_of_war.extend(self.battle_cards.iter_mut().map(|x| x.1));

        //Special Case 2: The player has less than four cards, but more than none. In this case, move all the battle_cards
        //to the spoils pile like usual, but when collecting additional spoils cards for the War itself, 
        //ensure to collect only enough such that they have one card remaining for the battle_cards.
        for winner in self.winners.iter(){
            //determine how many cards the player has left
            let war_card_count = std::cmp::min(self.players[*winner].cards.len(), 4) - 1;
            for _i in 0..war_card_count{
                println!("pushing {:?} times for player {:?}", war_card_count, *winner);
                if let Some(card) = self.players[*winner].cards.pop(){
                    self.spoils_of_war.push(card);
                }
            }
        }
    }

    fn ensure_player_has_last_card(&mut self){
        //Special Case #1 During War
        for winner in self.winners.iter(){
            println!("player index: {:?} length: {:?}", *winner, self.players[*winner].cards.len());
            if self.players[*winner].cards.len() < 1{
                //move the winning battle_card back to the player's deck
                println!("moving!");
                self.players[*winner].cards.push(self.battle_cards.remove(*winner).1);
            }
        }
    }

    fn award_player(&mut self){
        self.spoils_of_war.extend(self.battle_cards.iter_mut().map(|x| x.1));
        let mut new_deck: Vec<_> = self.spoils_of_war.drain(..).collect();
        new_deck.append(&mut self.players[self.winners[0]].cards);
        self.players[self.winners[0]].cards = new_deck;
    }

    fn play_cards(&mut self){
        for (i, player) in self.players.iter_mut().enumerate() {
            if let Some(card) = player.cards.pop() {
                self.battle_cards.push((i, card));
            }
        }
    }

    fn find_winners(&mut self) {
        //find the winning card indicies
        let biggest_winner: Vec<usize> = Vec::new();
        self.winners = self.battle_cards.iter().enumerate().fold(biggest_winner, |mut acc, x| {
            let index = x.0;
            match acc.first() {
                None => {
                    acc.push(self.battle_cards[index].0);
                    acc
                }
                Some(max) if self.battle_cards[*max].1.value < self.battle_cards[index].1.value => {
                    acc.clear();
                    acc.push(self.battle_cards[index].0);
                    acc
                }
                Some(max) if self.battle_cards[*max].1.value == self.battle_cards[index].1.value => {
                    acc.push(self.battle_cards[index].0);
                    acc
                }
                _ => acc,
            }
        });
    }

    fn check_win(&self) -> Option<usize> {
        let non_empty: Vec<usize> = self
            .players
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.cards.is_empty())
            .map(|(idx, _)| idx)
            .collect();

        match non_empty.as_slice() {
            [idx] => Some(*idx),
            _ => None,
        }
    }

    fn write_flush(&mut self, msg: &str) -> std::io::Result<()> {
        writeln!(self.writer, "{}", msg)?; //early return std::io::Result<()>
        self.writer.flush() //returns std::io::Result<()>
    }

    fn deal(&mut self) {
        let mut i = 0;
        let player_count = self.players.len();
        while let Some(card) = self.dealer.cards.pop() {
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

fn main() {
    let mut table = Table::new();

    let player_count: usize = loop {
        match table.initialize() {
            Ok(val) => break val,
            Err(e) => {
                println!("Invalid value: CardValue::{}", e)
            } //try again
        };
    };

    table.set_player_count(player_count);
    table.dealer.shuffle(&mut table.rng);
    table.deal();
    table.game_loop();
}

#[cfg(test)]
mod tests {

    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_table_new() {
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let table = Table::new_unit_test(rng, reader, writer);

        assert_eq!(table.dealer.cards.len(), 52);
        assert_eq!(table.players.len(), 0);
    }

    #[test]
    fn test_set_player_count() {
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
    fn test_deal() {
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

        let correct_players = vec![
            Deck {
                cards: vec![
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Two,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Five,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Eight,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Jack,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Ace,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Four,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Seven,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Ten,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::King,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Three,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Six,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Nine,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Queen,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Two,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Five,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Eight,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Jack,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Ace,
                    },
                ],
            },
            Deck {
                cards: vec![
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Three,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Six,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Nine,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Queen,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Two,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Five,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Eight,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Jack,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Ace,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Four,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Seven,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Ten,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::King,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Three,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Six,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Nine,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Queen,
                    },
                ],
            },
            Deck {
                cards: vec![
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Four,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Seven,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::Ten,
                    },
                    Card {
                        suit: Suit::Clubs,
                        value: CardValue::King,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Three,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Six,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Nine,
                    },
                    Card {
                        suit: Suit::Diamonds,
                        value: CardValue::Queen,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Two,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Five,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Eight,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Jack,
                    },
                    Card {
                        suit: Suit::Hearts,
                        value: CardValue::Ace,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Four,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Seven,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::Ten,
                    },
                    Card {
                        suit: Suit::Spades,
                        value: CardValue::King,
                    },
                ],
            },
        ];

        assert_eq!(table.players, correct_players);
    }

    #[test]
    fn test_check_win() {}

    #[test]
    fn test_play_cards() {
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);
        table.deal();

        table.play_cards();
        let correct_cards = vec![
            (
                0,
                Card {
                    suit: Suit::Spades,
                    value: CardValue::Ace,
                },
            ),
            (
                1,
                Card {
                    suit: Suit::Spades,
                    value: CardValue::Queen,
                },
            ),
            (
                2,
                Card {
                    suit: Suit::Spades,
                    value: CardValue::King,
                },
            ),
        ];
        assert_eq!(table.battle_cards, correct_cards);
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
        table.play_cards();
        table.find_winners();

        let correct_winners = vec![0];
        assert_eq!(table.winners, correct_winners);

        table.battle_cards.clear();
        for player in table.players.iter_mut() {
            player.shuffle(&mut table.rng);
        }
        table.play_cards();
        table.find_winners();

        let correct_winners = vec![2];
        assert_eq!(table.winners, correct_winners);

        table.battle_cards.clear();
        for player in table.players.iter_mut() {
            player.shuffle(&mut table.rng);
        }
        table.play_cards();
        table.find_winners();
        assert_eq!(table.winners, vec![1]);

        table.battle_cards.clear();
        table.players[0].cards.push(Card {
            suit: Suit::Spades,
            value: CardValue::King,
        });
        table.players[1].cards.push(Card {
            suit: Suit::Hearts,
            value: CardValue::King,
        });
        table.play_cards();
        table.find_winners();
        assert_eq!(table.winners, vec![0, 1]);

        table.battle_cards.clear();
        table.players[0].cards.push(Card {
            suit: Suit::Clubs,
            value: CardValue::Ace,
        });
        table.players[1].cards.push(Card {
            suit: Suit::Hearts,
            value: CardValue::Three,
        });
        table.players[2].cards.push(Card {
            suit: Suit::Clubs,
            value: CardValue::Three,
        });

        table.play_cards();
        table.find_winners();
        assert_eq!(table.winners, vec![0]);
    }

    #[test]
    fn test_award_player(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);
        table.players[0].cards.push(Card{suit: Suit::Clubs, value: CardValue::Ace});
        table.players[1].cards.push(Card{suit: Suit::Spades, value: CardValue::Jack});
        table.players[2].cards.push(Card{suit: Suit::Clubs, value: CardValue::Four});

        table.play_cards();
        table.find_winners();
        table.award_player();

        assert_eq!(table.players[0].cards, vec![Card{suit: Suit::Clubs, value: CardValue::Ace}, Card{suit: Suit::Spades, value: CardValue::Jack}, Card{suit: Suit::Clubs, value: CardValue::Four}]);
        assert_eq!(table.players[1].cards, Vec::<Card>::new());
        assert_eq!(table.players[2].cards, Vec::<Card>::new());
    }
    #[test]
    fn test_ensure_player_has_last_card(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        let player1 = Deck{cards: vec![
            Card{suit: Suit::Clubs, value: CardValue::Ace},
            Card{suit: Suit::Clubs, value: CardValue::King},
            Card{suit: Suit::Clubs, value: CardValue::Queen},
            Card{suit: Suit::Clubs, value: CardValue::Jack},
            ],
        };

        let player2 = Deck{cards: vec![
            Card{suit: Suit::Spades, value: CardValue::Ace},
            Card{suit: Suit::Spades, value: CardValue::King},
            Card{suit: Suit::Spades, value: CardValue::Queen},
            Card{suit: Suit::Spades, value: CardValue::Jack},
            ],
        };

        let player3 = Deck{cards: vec![
            Card{suit: Suit::Hearts, value: CardValue::Ace},
            ],
        };

        table.players.push(player1);
        table.players.push(player2);
        table.players.push(player3);

        table.play_cards();
        table.find_winners();

        let correct_battle_cards = vec![(0, Card{suit: Suit::Clubs, value: CardValue::Jack}), (1, Card{suit: Suit::Spades, value: CardValue::Jack}), (2, Card{suit: Suit::Hearts, value: CardValue::Ace})];
        assert_eq!(table.battle_cards, correct_battle_cards);

        table.ensure_player_has_last_card();

        let correct_battle_cards = vec![(0, Card{suit: Suit::Clubs, value: CardValue::Jack}), (1, Card{suit: Suit::Spades, value: CardValue::Jack})];
        assert_eq!(table.battle_cards, correct_battle_cards);
        assert_eq!(table.players[0].cards.len(), 3);
        assert_eq!(table.players[1].cards.len(), 3);
        assert_eq!(table.players[2].cards, vec![Card{suit: Suit::Hearts, value: CardValue::Ace}]);
    }

    #[test]
    fn test_build_spoils_of_war(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        let player1 = Deck{cards: vec![
            Card{suit: Suit::Clubs, value: CardValue::Ace},
            Card{suit: Suit::Clubs, value: CardValue::King},
            Card{suit: Suit::Clubs, value: CardValue::Queen},
            Card{suit: Suit::Clubs, value: CardValue::Jack},
            Card{suit: Suit::Clubs, value: CardValue::Ten},
            ],
        };

        let player2 = Deck{cards: vec![
            Card{suit: Suit::Spades, value: CardValue::Ace},
            Card{suit: Suit::Spades, value: CardValue::King},
            Card{suit: Suit::Spades, value: CardValue::Queen},
            Card{suit: Suit::Spades, value: CardValue::Ten},
            ],
        };

        let player3 = Deck{cards: vec![
            Card{suit: Suit::Hearts, value: CardValue::Ace},
            Card{suit: Suit::Hearts, value: CardValue::Ten},
            ],
        };

        table.players.push(player1);
        table.players.push(player2);
        table.players.push(player3);

        table.play_cards();
        table.find_winners();
        table.build_spoils_of_war();

        let correct_spoils = vec![
            Card{suit: Suit::Clubs, value: CardValue::Ten},
            Card{suit: Suit::Spades, value: CardValue::Ten},
            Card{suit: Suit::Hearts, value: CardValue::Ten},
            Card{suit: Suit::Clubs, value: CardValue::Jack},
            Card{suit: Suit::Clubs, value: CardValue::Queen},
            Card{suit: Suit::Clubs, value: CardValue::King},
            Card{suit: Suit::Spades, value: CardValue::Queen},
            Card{suit: Suit::Spades, value: CardValue::King},
        ];
        assert_eq!(table.spoils_of_war, correct_spoils);

        let correct_player1 = Deck{cards: vec![
            Card{suit: Suit::Clubs, value: CardValue::Ace},
            ],
        };

        let correct_player2 = Deck{cards: vec![
            Card{suit: Suit::Spades, value: CardValue::Ace},
            ],
        };

        let correct_player3 = Deck{cards: vec![
            Card{suit: Suit::Hearts, value: CardValue::Ace},
            ],
        };

        assert_eq!(table.players[0], correct_player1);
        assert_eq!(table.players[1], correct_player2);
        assert_eq!(table.players[2], correct_player3);
        
    }

    #[test]
    fn test_game_loop(){
        let rng = rand::rngs::StdRng::seed_from_u64(42);
        let input = b"\n";
        let reader = &input[..];
        let writer = Vec::new();
        let mut table = Table::new_unit_test(rng, reader, writer);

        table.set_player_count(3);

        let player1 = Deck{cards: vec![
            Card{suit: Suit::Clubs, value: CardValue::Ace},
            Card{suit: Suit::Clubs, value: CardValue::King},
            Card{suit: Suit::Clubs, value: CardValue::Queen},
            Card{suit: Suit::Clubs, value: CardValue::Jack},
            Card{suit: Suit::Clubs, value: CardValue::Ten},
            Card{suit: Suit::Clubs, value: CardValue::Nine},
            Card{suit: Suit::Clubs, value: CardValue::Eight},
            Card{suit: Suit::Clubs, value: CardValue::Seven},
            Card{suit: Suit::Clubs, value: CardValue::Six}
            ],
        };

        let player2 = Deck{cards: vec![
            Card{suit: Suit::Spades, value: CardValue::Ace},
            Card{suit: Suit::Spades, value: CardValue::King},
            Card{suit: Suit::Spades, value: CardValue::Queen},
            Card{suit: Suit::Spades, value: CardValue::Jack},
            Card{suit: Suit::Spades, value: CardValue::Ten},
            Card{suit: Suit::Spades, value: CardValue::Nine},
            Card{suit: Suit::Spades, value: CardValue::Eight},
            Card{suit: Suit::Spades, value: CardValue::Seven},
            Card{suit: Suit::Spades, value: CardValue::Six}
            ],
        };

        let player3 = Deck{cards: vec![
            Card{suit: Suit::Hearts, value: CardValue::Ace},
            Card{suit: Suit::Hearts, value: CardValue::King},
            Card{suit: Suit::Hearts, value: CardValue::Queen},
            Card{suit: Suit::Hearts, value: CardValue::Jack},
            Card{suit: Suit::Hearts, value: CardValue::Two},
            Card{suit: Suit::Hearts, value: CardValue::Three},
            Card{suit: Suit::Hearts, value: CardValue::Four},
            Card{suit: Suit::Hearts, value: CardValue::Seven},
            Card{suit: Suit::Hearts, value: CardValue::Six},
            ],
        };
    }
}
