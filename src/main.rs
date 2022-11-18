mod chess;
mod fen;
mod mover;

use std::io::Write;

use chess::{Board, Error};
use colored::*;

fn main() -> Result<(), Error> {
    let mut board = Board::default_board()?;

    let mut error: Option<String> = Option::None;

    loop {
        // clear screen
        print!("{}[2J", 27 as char);

        if error.is_some() {
            println!("\n{}\n", error.clone().unwrap().red());
        }

        // draw a chess board with file and ranks identifiers
        draw(&board);

        // print turn
        println!("\n{} to move:", board.turn().to_string().bold());
        print!("> ");

        // flush stdout
        std::io::stdout().flush().unwrap();

        // a move consists of 4 characters (e.g. e2e4)
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let mut cmd = input.split_whitespace().into_iter();

        match cmd.next() {
            // Exit commands
            Some("q") => break,
            Some("quit") => break,
            Some("exit") => break,

            // Save command, takes parameter of file
            Some("save") => {
                let filename = cmd.next().unwrap_or("game.txt");
                board.save(filename)?;
            }

            Some("load") => {
                let filename = cmd.next().unwrap_or("game.txt");
                board.load(filename)?;
            }

            Some(turn) => {
                error = match board.move_piece(turn) {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                }
            }
            _ => {}
        }
    }
    // parse turn
    Ok(())
}

fn draw(board: &Board) {
    println!("  ａｂｃｄｅｆｇｈ");
    for rank in 0..8 {
        let rank = 8 - rank;
        print!("{} ", rank);
        for file in 0..8 {
            let square_color = if (file + rank) % 2 == 0 {
                Color::White
            } else {
                Color::BrightBlue
            };
            let piece = board.get_piece(file, rank - 1);

            if piece.is_some() {
                let piece = piece.unwrap();
                print!(
                    "{}",
                    piece.to_string().color(Color::Black).on_color(square_color)
                );
            } else {
                print!("{}", " ".on_color(square_color));
            }

            print!("{}", " ".on_color(square_color));
        }
        println!(" {}", rank);
    }
    println!("  ａｂｃｄｅｆｇｈ");
}
