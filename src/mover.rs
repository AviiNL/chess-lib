use crate::chess::{Board, Class, Color, Error};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Move {
    pub from_file: usize,
    pub from_rank: usize,
    pub to_file: usize,
    pub to_rank: usize,
}

impl Move {
    pub fn new(from_file: usize, from_rank: usize, to_file: usize, to_rank: usize) -> Move {
        Move {
            from_file,
            from_rank,
            to_file,
            to_rank,
        }
    }

    pub fn distance(&self) -> usize {
        let file_distance = self.from_file as isize - self.to_file as isize;
        let rank_distance = self.from_rank as isize - self.to_rank as isize;

        (file_distance.abs() + rank_distance.abs()) as usize
    }

    pub fn validate(&self, board: &Board) -> Result<(), Error> {
        let piece = board.get_piece(self.from_file, self.from_rank);

        // There is no piece on the square
        if piece.is_none() {
            return Err(Error::InvalidMove("No piece on square".to_string()));
        }

        let piece = piece.unwrap();

        // Not our piece
        if piece.color != board.turn() {
            return Err(Error::InvalidMove("Not your piece".to_string()));
        }

        // class independent validation
        match piece.class {
            Class::Pawn => self.validate_pawn(board),
            Class::Knight => self.validate_knight(board),
            Class::Bishop => self.validate_bishop(board),
            Class::Rook => self.validate_rook(board),
            Class::Queen => self.validate_queen(board),
            Class::King => self.validate_king(board),
        }?;

        // capture check
        let target = board.get_piece(self.to_file, self.to_rank);

        // Trying to capture our own piece
        if let Some(t) = target {
            if t.color == board.turn() {
                return Err(Error::InvalidMove(
                    "Can't capture your own piece".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Move {
    fn validate_pawn(&self, board: &Board) -> Result<(), Error> {
        let piece = board.get_piece(self.from_file, self.from_rank).unwrap();

        // Pawn can only move forward
        if piece.color == Color::White && self.to_rank < self.from_rank {
            return Err(Error::InvalidMove("Pawn can only move forward".to_string()));
        }

        if piece.color == Color::Black && self.to_rank > self.from_rank {
            return Err(Error::InvalidMove("Pawn can only move forward".to_string()));
        }

        let from_file: i32 = self.from_file as i32;
        let from_rank: i32 = self.from_rank as i32;

        let to_file: i32 = self.to_file as i32;
        let to_rank: i32 = self.to_rank as i32;

        // Pawn can only move one square forward, unless it is the first move
        if piece.moves == 0 {
            if (to_rank - from_rank).abs() > 2 || (to_rank - from_rank).abs() < 1 {
                return Err(Error::InvalidMove(
                    format!("Pawn can only move one or two squares forward on the first move, attempted to move {} squares", (to_rank - from_rank).abs()).to_string(),
                ));
            }
        } else {
            if (to_rank - from_rank).abs() != 1 {
                return Err(Error::InvalidMove(
                    "Pawn can only move one square forward".to_string(),
                ));
            }
        }

        let target = match board.get_piece(self.to_file, self.to_rank) {
            Some(t) => {
                if t.color == piece.color {
                    None
                } else {
                    Some(t)
                }
            }
            None => None,
        };

        // Pawn can only move one square sideways if capturing target
        if target.is_some() {
            if (to_file - from_file).abs() != 1 {
                return Err(Error::InvalidMove(
                    "Pawn can only capture diagonally".to_string(),
                ));
            }
        } else {
            // if the destination square is board.en_passant, then we are capturing en passant
            if board.is_en_passant(self.to_file, self.to_rank) {
                return Ok(());
            }

            if (to_file - from_file).abs() != 0 {
                return Err(Error::InvalidMove(
                    "Pawn can not move diagonally".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Move {
    pub fn validate_knight(&self, _: &Board) -> Result<(), Error> {
        let from_file: i32 = self.from_file as i32;
        let from_rank: i32 = self.from_rank as i32;

        let to_file: i32 = self.to_file as i32;
        let to_rank: i32 = self.to_rank as i32;

        // Knight can only move two squares forward and one square sideways, or two squares sideways and one square forward
        if (to_file - from_file).abs() == 2 && (to_rank - from_rank).abs() == 1 {
            return Ok(());
        }

        if (to_file - from_file).abs() == 1 && (to_rank - from_rank).abs() == 2 {
            return Ok(());
        }

        Err(Error::InvalidMove(
            "Knight can only move two squares forward and one square sideways, or two squares sideways and one square forward".to_string(),
        ))
    }
}

impl Move {
    pub fn validate_bishop(&self, board: &Board) -> Result<(), Error> {
        let from_file: i32 = self.from_file as i32;
        let from_rank: i32 = self.from_rank as i32;

        let to_file: i32 = self.to_file as i32;
        let to_rank: i32 = self.to_rank as i32;

        // Bishop can only move diagonally
        if (to_file - from_file).abs() != (to_rank - from_rank).abs() {
            return Err(Error::InvalidMove(
                "Bishop can only move diagonally".to_string(),
            ));
        }

        // Check if there are any pieces in the way
        let file_direction = if to_file > from_file { 1 } else { -1 };

        let rank_direction = if to_rank > from_rank { 1 } else { -1 };

        let mut file = from_file + file_direction;
        let mut rank = from_rank + rank_direction;

        while file != to_file && rank != to_rank {
            if let Some(_) = board.get_piece(file as usize, rank as usize) {
                return Err(Error::InvalidMove(
                    "Bishop can not move through pieces".to_string(),
                ));
            }

            file += file_direction;
            rank += rank_direction;
        }

        Ok(())
    }
}

impl Move {
    pub fn validate_rook(&self, board: &Board) -> Result<(), Error> {
        let from_file: i32 = self.from_file as i32;
        let from_rank: i32 = self.from_rank as i32;

        let to_file: i32 = self.to_file as i32;
        let to_rank: i32 = self.to_rank as i32;

        // Rook can only move horizontally or vertically
        if from_file != to_file && from_rank != to_rank {
            return Err(Error::InvalidMove(
                "Rook can only move horizontally or vertically".to_string(),
            ));
        }

        // Check if there are any pieces in the way
        if from_file == to_file {
            let direction = if to_rank > from_rank { 1 } else { -1 };

            let mut rank = from_rank + direction;

            while rank != to_rank {
                if let Some(_) = board.get_piece(from_file as usize, rank as usize) {
                    return Err(Error::InvalidMove(
                        "Rook can not move through pieces".to_string(),
                    ));
                }

                rank += direction;
            }
        } else {
            let direction = if to_file > from_file { 1 } else { -1 };

            let mut file = from_file + direction;

            while file != to_file {
                if let Some(_) = board.get_piece(file as usize, from_rank as usize) {
                    return Err(Error::InvalidMove(
                        "Rook can not move through pieces".to_string(),
                    ));
                }

                file += direction;
            }
        }

        Ok(())
    }
}

impl Move {
    pub fn validate_queen(&self, board: &Board) -> Result<(), Error> {
        let from_file: i32 = self.from_file as i32;
        let from_rank: i32 = self.from_rank as i32;

        let to_file: i32 = self.to_file as i32;
        let to_rank: i32 = self.to_rank as i32;

        // Queen can only move horizontally, vertically, or diagonally
        if from_file != to_file
            && from_rank != to_rank
            && (to_file - from_file).abs() != (to_rank - from_rank).abs()
        {
            return Err(Error::InvalidMove(
                "Queen can only move horizontally, vertically, or diagonally".to_string(),
            ));
        }

        // Check if there are any pieces in the way
        if from_file == to_file {
            let direction = if to_rank > from_rank { 1 } else { -1 };

            let mut rank = from_rank + direction;

            while rank != to_rank {
                if let Some(_) = board.get_piece(from_file as usize, rank as usize) {
                    return Err(Error::InvalidMove(
                        "Queen can not move through pieces".to_string(),
                    ));
                }

                rank += direction;
            }
        } else if from_rank == to_rank {
            let direction = if to_file > from_file { 1 } else { -1 };

            let mut file = from_file + direction;

            while file != to_file {
                if let Some(_) = board.get_piece(file as usize, from_rank as usize) {
                    return Err(Error::InvalidMove(
                        "Queen can not move through pieces".to_string(),
                    ));
                }

                file += direction;
            }
        } else {
            let file_direction = if to_file > from_file { 1 } else { -1 };

            let rank_direction = if to_rank > from_rank { 1 } else { -1 };

            let mut file = from_file + file_direction;
            let mut rank = from_rank + rank_direction;

            while file != to_file && rank != to_rank {
                if let Some(_) = board.get_piece(file as usize, rank as usize) {
                    return Err(Error::InvalidMove(
                        "Queen can not move through pieces".to_string(),
                    ));
                }

                file += file_direction;
                rank += rank_direction;
            }
        }

        Ok(())
    }
}

impl Move {
    pub fn validate_king(&self, board: &Board) -> Result<(), Error> {
        let piece = board.get_piece(self.from_file, self.from_rank).unwrap();

        let from_file: i32 = self.from_file as i32;
        let from_rank: i32 = self.from_rank as i32;

        let to_file: i32 = self.to_file as i32;
        let to_rank: i32 = self.to_rank as i32;

        // Check if attempting to move on kingside (white)
        if piece.color == Color::White
            && piece.moves == 0
            && from_file == 4
            && from_rank == 0
            && to_file == 6
            && to_rank == 0
        {
            if let Some(rook) = board.get_piece(7, 0) {
                if rook.class == Class::Rook && rook.color == Color::White && rook.moves == 0 {
                    if board.get_piece(5, 0).is_none() && board.get_piece(6, 0).is_none() {
                        return Ok(());
                    }
                }
            }
        }

        // Check if attempting to move on queenside (white)
        if piece.color == Color::White
            && piece.moves == 0
            && from_file == 4
            && from_rank == 0
            && to_file == 2
            && to_rank == 0
        {
            if let Some(rook) = board.get_piece(0, 0) {
                if rook.class == Class::Rook && rook.color == Color::White && rook.moves == 0 {
                    if board.get_piece(1, 0).is_none()
                        && board.get_piece(2, 0).is_none()
                        && board.get_piece(3, 0).is_none()
                    {
                        return Ok(());
                    }
                }
            }
        }

        // Check if attempting to move on kingside (black)
        if piece.color == Color::Black
            && piece.moves == 0
            && from_file == 4
            && from_rank == 7
            && to_file == 6
            && to_rank == 7
        {
            if let Some(rook) = board.get_piece(7, 7) {
                if rook.class == Class::Rook && rook.color == Color::Black && rook.moves == 0 {
                    if board.get_piece(5, 7).is_none() && board.get_piece(6, 7).is_none() {
                        return Ok(());
                    }
                }
            }
        }

        // Check if attempting to move on queenside (black)
        if piece.color == Color::Black
            && piece.moves == 0
            && from_file == 4
            && from_rank == 7
            && to_file == 2
            && to_rank == 7
        {
            if let Some(rook) = board.get_piece(0, 7) {
                if rook.class == Class::Rook && rook.color == Color::Black && rook.moves == 0 {
                    if board.get_piece(1, 7).is_none()
                        && board.get_piece(2, 7).is_none()
                        && board.get_piece(3, 7).is_none()
                    {
                        return Ok(());
                    }
                }
            }
        }

        if (to_file - from_file).abs() > 1 || (to_rank - from_rank).abs() > 1 {
            return Err(Error::InvalidMove(
                "King can only move one square in any direction".to_string(),
            ));
        }

        Ok(())
    }
}

impl TryFrom<&str> for Move {
    type Error = crate::chess::Error;

    fn try_from(m: &str) -> Result<Move, Error> {
        let m = m.to_lowercase();

        let from_file = m.chars().nth(0).unwrap() as usize - 97;
        let from_rank = m.chars().nth(1).unwrap() as usize - 49;
        let to_file = m.chars().nth(2).unwrap() as usize - 97;
        let to_rank = m.chars().nth(3).unwrap() as usize - 49;

        // ensure that the move is within the bounds of the board
        if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
            return Err(Error::InvalidMove("Move is out of bounds".to_string()));
        }

        Ok(Move {
            from_file,
            from_rank,
            to_file,
            to_rank,
        })
    }
}

impl From<Move> for String {
    fn from(m: Move) -> String {
        let from_file = (m.from_file + 97) as u8 as char;
        let from_rank = (m.from_rank + 49) as u8 as char;
        let to_file = (m.to_file + 97) as u8 as char;
        let to_rank = (m.to_rank + 49) as u8 as char;

        format!("{}{}{}{}", from_file, from_rank, to_file, to_rank)
    }
}

impl ToString for Move {
    fn to_string(&self) -> String {
        let string = String::from(*self);

        string
    }
}
