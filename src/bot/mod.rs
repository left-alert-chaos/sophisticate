//!# bot
//!This module holds code to wrap search and eval and move gen into an accessible API. It also holds
//!the UCI implementation.

mod uci;
use crate::evaluation::SearchTree;
use crate::physical::*;
use std::io;
pub use uci::*;

//TODO: add config options
#[allow(dead_code)]
struct EngineOptions {}

#[allow(dead_code)]
impl EngineOptions {
    fn new() -> Self {
        Self {}
    }
}

///# Ocelot
///The whole reason we're here. This struct keeps track of game state and uses SearchTrees to
///generate the best move for the player the bot is assigned.
pub struct Ocelot {
    pub(crate) board: Board,
    pub(crate) depth: i32,
    ponder: Option<Box<dyn Action>>,
    allowed_time: f64,
    evaluation: f64,
}

impl Ocelot {
    pub fn new(board: &Board, depth: i32, allowed_time: f64) -> Self {
        Self {
            board: board.duplicate(),
            depth,
            ponder: None,
            allowed_time,
            evaluation: 0.0,
        }
    }

    ///This tries to get the best move with a SearchTree. If it can't find one, it returns the first
    ///legal one.
    ///Remember that this doesn't apply the move, just generates it.
    pub fn safe_best_move(&mut self) -> Box<dyn Action> {
        let mut tree = SearchTree::new(&self.board, self.depth, self.allowed_time);
        tree.safe_best_move()
    }

    pub fn perform_on_self(&mut self, mut action: Box<dyn Action>) {
        action.perform_on(&mut self.board);
    }

    ///Main engine loop. Get input, find move, etc
    pub fn uci_loop(&mut self) {
        loop {
            let mut input = String::new();
            let _ = io::stdin().read_line(&mut input);

            //interpret command and determine whether to end loop
            if !self.interpret_uci(input) {
                break;
            }
        }
    }

    //returns whether to continue loop
    fn interpret_uci(&mut self, input: String) -> bool {
        let mut words = input.split_whitespace();

        //get command
        let Some(command) = words.nth(0) else {
            return true;
        };

        let command_tail = input.split_at(command.len() + 1).1;

        //main decision tree
        match command {
            "quit" => return false,
            "uci" => {
                println!("uciok");
            }
            "ucinewgame" => {}
            "isready" => {
                println!("readyok");
            } //TODO: wait for background search to finish, if I ever implement
            //background search
            "setoption" => {} //TODO: Implement option setting
            "position" => {
                self.position(command_tail);
            }
            "go" => {
                self.parsed_go(command_tail);
            }
            "ponderhit" => {
                //perform pondered move
                if let Some(pondered) = &mut self.ponder {
                    pondered.perform_on(&mut self.board);
                }
            }
            "stop" => {} //TODO: If background eval is ever implemented, use this to kill it
            "d" => {
                println!("{}", self.board.draw());
            }
            "eval" => {
                println!("{}", self.evaluation);
            }
            _ => {
                eprintln!("Ocelot::interpret_uci(): Command {command} isn't implemented.");
            }
        }

        true
    }

    fn parsed_go(&mut self, command: &str) {
        //attempt to generate go. if failed, use default
        let go = match Go::parse(command.to_string(), &mut self.board) {
            Ok(go) => go,
            Err(_) => Go::new(),
        };

        //calculate allowed time
        let go_time = if self.board.turn == board::Color::White {
            go.wtime
        } else {
            go.btime
        };
        let allowed_time = match go_time {
            Some(time) => self.allowed_time_for_search(time),
            None => f64::INFINITY,
        };

        println!("Ocelot::parsed_go(): allowed_time is {allowed_time}");

        //actually calculate best move
        let mut tree = SearchTree::new(&self.board, self.depth, allowed_time);
        let mut best_move = tree.safe_best_move();
        let maybe_best_position = tree.root.best_child;

        //set evaluation
        if let Some(value) = tree.root.best_value {
            self.evaluation = value;
        }

        //get pondered move
        let ponder = if let Some(best_pos) = *maybe_best_position {
            if let Some(ponder) = best_pos.best_move {
                ponder.generate()
            } else {
                String::from("0000")
            }
        } else {
            String::from("0000")
        };

        //this might not be a great idea
        best_move.perform_on(&mut self.board);
        println!("bestmove {} ponder {ponder}", best_move.generate());
    }

    fn position(&mut self, repr: &str) {
        let repr = repr.replace("fen ", "");
        let result = Board::parse(repr.to_string());
        if let Ok(board) = result {
            self.board = board;
        }

        //get listed moves
        if repr.contains("moves") {
            let mut halves = repr.split("moves");

            let move_reprs = halves.nth(1).unwrap().split_whitespace();

            for move_repr in move_reprs {
                let action =
                    uci::parse_action(move_repr.to_string().trim().to_string(), &mut self.board);
                self.perform_on_self(action);
            }
        }
    }

    //questionable-at-best time allocation
    //designed to allow more time at then end than at the beginning of the game.
    fn allowed_time_for_search(&self, time: u64) -> f64 {
        let inverse_priority = 100 - self.board.round as u64;
        let allowed_time = (time as f64 / inverse_priority as f64) * 2.5;
        println!(
            "Ocelot::allowed_time_for_search(): Returning {allowed_time} due to inverse_priority {inverse_priority} and round {}",
            self.board.round
        );
        allowed_time
    }
}
