//!# go
//!This module holds code to parse a `go` UCI command.

use crate::bot::uci::safe_parse_action;
use crate::physical::*;

#[derive(Default, Debug)]
pub struct Go {
    pub(crate) cutoff: SearchCutoff,
    pub(crate) searchmoves: Vec<Box<dyn Action>>,
    pub(crate) wtime: Option<u64>,
    pub(crate) btime: Option<u64>,
    pub(crate) winc: Option<u64>,
    pub(crate) binc: Option<u64>,
    pub(crate) movestogo: Option<u64>,
    pub(crate) movetime: Option<u64>,
}

impl Go {
    pub fn parse(mut command: String, board: &mut Board) -> Result<Self, ()> {
        command = command.replace("go ", "");
        let mut words = command.split_whitespace();

        let mut go = Self {
            ..Default::default()
        };

        //find cutoff
        let mut finished = true;
        let mut cutoff = SearchCutoff::Infinite;
        let mut unit: &str = "";
        while let Some(word) = words.next() {
            match word {
                "infinite" => {
                    cutoff = SearchCutoff::Infinite;
                }
                "depth" | "nodes" | "mate" => unit = word,
                _ => {
                    if let Ok(num) = word.to_string().parse::<i32>() {
                        match unit {
                            "depth" => cutoff = SearchCutoff::Depth(num),
                            "nodes" => cutoff = SearchCutoff::Nodes(num),
                            "mate" => cutoff = SearchCutoff::Mate(num),
                            _ => continue,
                        }
                        println!("Set cutoff number to {num}");
                    }

                    finished = false;
                    break;
                }
            }
        }

        go.cutoff = cutoff;

        if finished {
            return Ok(go);
        }

        //find searchmoves
        finished = true;
        let mut searchmoves = Vec::new();
        let _ = words.next();
        let mut searchmoves_words = words.clone();
        while let Some(m) = searchmoves_words.next() {
            if let Ok(action) = safe_parse_action(m.to_string(), board) {
                searchmoves.push(action);
            } else {
                finished = false;
                break;
            }
        }

        //advance main iterator enough
        for _ in 0..searchmoves.len() {
            let _ = words.next();
        }

        go.searchmoves = searchmoves;

        if finished {
            return Ok(go);
        }

        //The rest of the arguments follow the syntax "vname <value>", so we can use one large loop
        //This advances 2 words at a time- a key and a value
        while let Some(key) = words.next() {
            //get value. if there is no value, something's wrong.
            let Some(value) = words.next() else {
                return Err(());
            };

            //parse value to number
            let Ok(num) = value.parse::<u64>() else {
                return Err(());
            };

            match key {
                "wtime" => go.wtime = Some(num),
                "btime" => go.btime = Some(num),
                "winc" => go.winc = Some(num),
                "binc" => go.binc = Some(num),
                "movestogo" => go.movestogo = Some(num),
                "movetime" => go.movetime = Some(num),
                _ => continue,
            }
        }

        Ok(go)
    }

    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum SearchCutoff {
    #[default]
    Infinite, //only time limit applies
    Mate(i32),
    Nodes(i32),
    Depth(i32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_go() {
        let go = Go::parse(
            String::from(
                "go nodes 24 searchmoves e2e4 d2d3 d2d4 wtime 1200000 btime 1200000 winc 1200000",
            ),
            &mut Default::default(),
        )
        .unwrap();

        assert_eq!(go.searchmoves.len(), 3);
        assert_eq!(go.wtime, Some(1200000));
        assert_eq!(go.btime, Some(1200000));
        assert_eq!(go.winc, Some(1200000));
        assert_eq!(go.cutoff, SearchCutoff::Nodes(24));
    }

    #[test]
    //parse() should return an error
    fn parse_bad_go_command() {
        let go = Go::parse(
            String::from(
                "go infinite searchmoves nope not today wtime not_a_number btime also_not_a_number",
            ),
            &mut Default::default(),
        );
        assert!(go.is_err());
    }
}
