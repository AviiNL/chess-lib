use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

use crate::{fen::ToFen, mover::Move};

pub const DEFAULT_BOARD: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub enum Error {
    InvalidInput,
    InvalidFen(String),
    InvalidMove(String),
    SaveFailed(String),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::SaveFailed(e.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInput => write!(f, "Invalid input"),
            Error::InvalidFen(fen) => write!(f, "Invalid FEN: {}", fen),
            Error::InvalidMove(m) => write!(f, "Invalid move: {}", m),
            Error::SaveFailed(file) => write!(f, "Failed to save game to file: {}", file),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Class {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl ToString for Color {
    fn to_string(&self) -> String {
        match self {
            Color::White => "white".to_string(),
            Color::Black => "black".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Piece {
    pub class: Class,
    pub color: Color,
    pub moves: usize,
}

impl Piece {
    pub fn new(class: Class, color: Color) -> Piece {
        Piece {
            class,
            color,
            moves: 0,
        }
    }
}

impl ToFen for Piece {
    fn to_fen(&self) -> String {
        match self.class {
            Class::Pawn => match self.color {
                Color::White => "P".to_string(),
                Color::Black => "p".to_string(),
            },
            Class::Knight => match self.color {
                Color::White => "N".to_string(),
                Color::Black => "n".to_string(),
            },
            Class::Bishop => match self.color {
                Color::White => "B".to_string(),
                Color::Black => "b".to_string(),
            },
            Class::Rook => match self.color {
                Color::White => "R".to_string(),
                Color::Black => "r".to_string(),
            },
            Class::Queen => match self.color {
                Color::White => "Q".to_string(),
                Color::Black => "q".to_string(),
            },
            Class::King => match self.color {
                Color::White => "K".to_string(),
                Color::Black => "k".to_string(),
            },
        }
    }
}

impl ToString for Piece {
    fn to_string(&self) -> String {
        match self.class {
            Class::Pawn => match self.color {
                Color::White => "♙".to_string(),
                Color::Black => "♟".to_string(),
            },
            Class::Knight => match self.color {
                Color::White => "♘".to_string(),
                Color::Black => "♞".to_string(),
            },
            Class::Bishop => match self.color {
                Color::White => "♗".to_string(),
                Color::Black => "♝".to_string(),
            },
            Class::Rook => match self.color {
                Color::White => "♖".to_string(),
                Color::Black => "♜".to_string(),
            },
            Class::Queen => match self.color {
                Color::White => "♕".to_string(),
                Color::Black => "♛".to_string(),
            },
            Class::King => match self.color {
                Color::White => "♔".to_string(),
                Color::Black => "♚".to_string(),
            },
        }
    }
}

pub struct Board {
    pieces: [[Option<Piece>; 8]; 8],
    turn: Color,
    captured: Vec<Piece>,
    moves: Vec<String>,

    white_can_castle_kingside: bool,
    white_can_castle_queenside: bool,

    black_can_castle_kingside: bool,
    black_can_castle_queenside: bool,

    en_passant: Option<(usize, usize)>,

    halfmove_clock: usize,
    fullmove_number: usize,
}

impl Board {
    /// Creates an empty board
    pub fn new() -> Result<Board, Error> {
        let board = Board {
            pieces: [[Option::None; 8]; 8],
            turn: Color::White,
            captured: Vec::new(),
            moves: Vec::new(),
            white_can_castle_kingside: true,
            white_can_castle_queenside: true,
            black_can_castle_kingside: true,
            black_can_castle_queenside: true,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };

        Ok(board)
    }

    /// Creates a board with default pieces
    pub fn default_board() -> Result<Board, Error> {
        let mut board = Board::new()?;
        board.from_fen(DEFAULT_BOARD)?;
        Ok(board)
    }

    fn clear_piece(&mut self, file: usize, rank: usize) {
        self.pieces[file][rank] = Option::None;
    }

    fn set_piece(&mut self, piece: Piece, file: usize, rank: usize) {
        self.pieces[file][rank] = Option::Some(piece);
    }

    pub fn get_piece(&self, file: usize, rank: usize) -> Option<Piece> {
        self.pieces[file][rank]
    }

    pub fn is_en_passant(&self, file: usize, rank: usize) -> bool {
        match self.en_passant {
            Some((f, r)) => file == f && rank == r,
            None => false,
        }
    }

    // a function that returns who's turn it is
    pub fn turn(&self) -> Color {
        self.turn
    }

    pub fn move_piece(&mut self, data: &str) -> Result<(), Error> {
        let data = data.trim();

        let m: Move = data.try_into()?;

        // validate move against board status
        m.validate(&self)?;

        self.halfmove_clock += 1;

        let mut piece = self.get_piece(m.from_file, m.from_rank).unwrap();

        // check if the destination is an en passnt capture
        if self.is_en_passant(m.to_file, m.to_rank) {
            let rank = match piece.color {
                Color::White => m.to_rank - 1,
                Color::Black => m.to_rank + 1,
            };

            self.halfmove_clock = 0;
            self.captured.push(self.get_piece(m.to_file, rank).unwrap());
            self.clear_piece(m.to_file, rank);
        }

        let target = self.get_piece(m.to_file, m.to_rank);

        if let Some(capture) = target {
            self.halfmove_clock = 0;
            self.captured.push(capture);
        }

        // set en passant if pawn moves two spaces
        if piece.class == Class::Pawn {
            self.halfmove_clock = 0;
            if m.distance() == 2 {
                let rank = if piece.color == Color::White {
                    m.to_rank - 1
                } else {
                    m.to_rank + 1
                };

                self.en_passant = Some((m.to_file, rank));
            } else {
                self.en_passant = None;
            }
        }

        // check if the move is a castle
        if piece.class == Class::King {
            if m.to_file == 6 {
                let rook = self.get_piece(7, m.to_rank).unwrap();
                self.set_piece(rook, 5, m.to_rank);
                self.clear_piece(7, m.to_rank);
            } else if m.to_file == 2 {
                let rook = self.get_piece(0, m.to_rank).unwrap();
                self.set_piece(rook, 3, m.to_rank);
                self.clear_piece(0, m.to_rank);
            }
        }

        piece.moves += 1;
        self.set_piece(piece, m.to_file, m.to_rank);
        self.clear_piece(m.from_file, m.from_rank);

        self.moves.push(data.to_string());

        // switch turn
        self.turn = match self.turn {
            Color::White => Color::Black,
            Color::Black => {
                self.fullmove_number += 1;
                Color::White
            }
        };

        Ok(())
    }

    pub fn from_fen(&mut self, data: &str) -> Result<(), Error> {
        *self = Board::new()?;
        let mut file = 0;
        let mut rank = 7;

        // split by spaces
        let parts: Vec<&str> = data.split(" ").collect();
        let mut parts = parts.iter();

        let moves = match parts.next() {
            Some(moves) => moves,
            None => return Err(Error::InvalidFen("missing moves".to_string())),
        };

        self.turn = match parts.next() {
            Some(start) => match start {
                &"w" => Color::White,
                &"b" => Color::Black,
                _ => return Err(Error::InvalidFen("invalid start".to_string())),
            },
            None => return Err(Error::InvalidFen("missing start".to_string())),
        };

        match parts.next() {
            Some(castling) => {
                self.white_can_castle_kingside = castling.contains("K");
                self.white_can_castle_queenside = castling.contains("Q");
                self.black_can_castle_kingside = castling.contains("k");
                self.black_can_castle_queenside = castling.contains("q");
            }
            None => return Err(Error::InvalidFen("missing castling".to_string())),
        };

        match parts.next() {
            Some(en_passant) => {
                if en_passant != &"-" {
                    let file = match en_passant.chars().nth(0) {
                        Some(file) => match file {
                            'a' => 0,
                            'b' => 1,
                            'c' => 2,
                            'd' => 3,
                            'e' => 4,
                            'f' => 5,
                            'g' => 6,
                            'h' => 7,
                            _ => {
                                return Err(Error::InvalidFen(
                                    "invalid en passant file".to_string(),
                                ))
                            }
                        },
                        None => {
                            return Err(Error::InvalidFen("missing en passant file".to_string()))
                        }
                    };

                    let rank = match en_passant.chars().nth(1) {
                        Some(rank) => match rank {
                            '3' => 2,
                            '6' => 5,
                            _ => {
                                return Err(Error::InvalidFen(
                                    "invalid en passant rank".to_string(),
                                ))
                            }
                        },
                        None => {
                            return Err(Error::InvalidFen("missing en passant rank".to_string()))
                        }
                    };

                    self.en_passant = Some((file, rank))
                }
            }
            None => return Err(Error::InvalidFen("missing en passant".to_string())),
        };

        self.halfmove_clock = match parts.next() {
            Some(halfmove) => {
                let halfmove = match halfmove.parse::<usize>() {
                    Ok(halfmove) => halfmove,
                    Err(_) => return Err(Error::InvalidFen("invalid halfmove clock".to_string())),
                };
                // if halfmove > 100 {
                //     return Err(Error::InvalidFen("invalid halfmove clock".to_string()));
                // }
                halfmove
            }
            None => return Err(Error::InvalidFen("missing halfmove".to_string())),
        };

        self.fullmove_number = match parts.next() {
            Some(fullmove) => {
                let fullmove = match fullmove.parse::<usize>() {
                    Ok(fullmove) => fullmove,
                    Err(_) => return Err(Error::InvalidFen("invalid fullmove number".to_string())),
                };
                // if fullmove > 100 {
                //     return Err(Error::InvalidFen("invalid fullmove number".to_string()));
                // }
                fullmove
            }
            None => return Err(Error::InvalidFen("missing fullmove".to_string())),
        };

        // split by slashes
        let rows: Vec<&str> = moves.split("/").collect();

        for row in rows {
            for c in row.chars() {
                if c.is_ascii_digit() {
                    let count = c.to_string().parse::<usize>().unwrap();
                    file += count;
                    continue;
                }

                let piece = match c {
                    'P' => Piece::new(Class::Pawn, Color::White),
                    'N' => Piece::new(Class::Knight, Color::White),
                    'B' => Piece::new(Class::Bishop, Color::White),
                    'R' => Piece::new(Class::Rook, Color::White),
                    'Q' => Piece::new(Class::Queen, Color::White),
                    'K' => Piece::new(Class::King, Color::White),
                    'p' => Piece::new(Class::Pawn, Color::Black),
                    'n' => Piece::new(Class::Knight, Color::Black),
                    'b' => Piece::new(Class::Bishop, Color::Black),
                    'r' => Piece::new(Class::Rook, Color::Black),
                    'q' => Piece::new(Class::Queen, Color::Black),
                    'k' => Piece::new(Class::King, Color::Black),
                    _ => return Err(Error::InvalidFen("invalid piece".to_string())),
                };

                self.set_piece(piece, file, rank);

                file += 1;
            }

            file = 0;
            if rank > 0 {
                rank -= 1;
            }
        }

        Ok(())
    }

    // pub fn to_fen(&self) -> String {
    //     // Not enough recorded data yet to implement this
    //     let mut fen = String::new();
    //     // for row in self.pieces.iter() {
    //     //     let mut empty = 0;
    //     //     for piece in row.iter() {
    //     //         if piece.is_none() {
    //     //             empty += 1;
    //     //         } else {
    //     //             if empty > 0 {
    //     //                 fen.push_str(&empty.to_string());
    //     //                 empty = 0;
    //     //             }
    //     //             fen.push_str(&piece.unwrap().to_string());
    //     //         }
    //     //     }
    //     //     if empty > 0 {
    //     //         fen.push_str(&empty.to_string());
    //     //     }
    //     //     fen.push('/');
    //     // }
    //     // fen.pop();
    //     fen
    // }
}

impl Board {
    pub fn reset(&mut self) -> Result<(), Error> {
        *self = Board::new()?;
        Ok(())
    }

    pub fn save(&self, filename: &str) -> Result<(), Error> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        // write all the moves, space seperated into file
        for m in self.moves.iter() {
            writer.write_all(m.to_string().as_bytes())?;
            writer.write_all(b" ")?;
        }

        Ok(())
    }

    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);

        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let contents = contents.trim();

        let moves: Vec<_> = contents.split(" ").collect();

        // reset the board
        self.reset()?;

        for m in moves {
            let m = m.trim();
            if m.len() > 0 {
                self.move_piece(m)?;
            }
        }

        Ok(())
    }
}
