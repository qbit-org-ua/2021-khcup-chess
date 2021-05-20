use std::io::BufRead;

use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChessBoardPosition {
    pub row: u8,
    pub column: u8,
}

impl std::fmt::Display for ChessBoardPosition {
    /// ```
    /// use std::str::FromStr;
    /// use chess_interactor::ChessBoardPosition;
    /// assert_eq!(format!("{}", ChessBoardPosition::from_str("a1").unwrap()), "a1");
    /// assert_eq!(format!("{}", ChessBoardPosition::from_str("b1").unwrap()), "b1");
    /// assert_eq!(format!("{}", ChessBoardPosition::from_str("a8").unwrap()), "a8");
    /// assert_eq!(format!("{}", ChessBoardPosition::from_str("h8").unwrap()), "h8");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.column + b'a') as char, self.row + 1)
    }
}

impl std::str::FromStr for ChessBoardPosition {
    type Err = &'static str;

    /// ```
    /// use std::str::FromStr;
    /// use chess_interactor::ChessBoardPosition;
    /// assert!(matches!(ChessBoardPosition::from_str("a1").unwrap(), ChessBoardPosition { column: 0, row: 0 }));
    /// assert!(matches!(ChessBoardPosition::from_str("a8").unwrap(), ChessBoardPosition { column: 0, row: 7 }));
    /// assert!(matches!(ChessBoardPosition::from_str("d2").unwrap(), ChessBoardPosition { column: 3, row: 1 }));
    /// assert!(matches!(ChessBoardPosition::from_str("h1").unwrap(), ChessBoardPosition { column: 7, row: 0 }));
    /// assert!(matches!(ChessBoardPosition::from_str("h8").unwrap(), ChessBoardPosition { column: 7, row: 7 }));
    /// assert_eq!(ChessBoardPosition::from_str("asd").unwrap_err(), "invalid length");
    /// assert_eq!(ChessBoardPosition::from_str(" 1").unwrap_err(), "invalid column");
    /// assert_eq!(ChessBoardPosition::from_str("  ").unwrap_err(), "invalid column");
    /// assert_eq!(ChessBoardPosition::from_str("as").unwrap_err(), "invalid row");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err("invalid length");
        }
        let s = s.as_bytes();
        let column = if let column @ b'a'..=b'h' = s[0] {
            column - b'a'
        } else {
            return Err("invalid column");
        };
        let row = if let row @ b'1'..=b'8' = s[1] {
            row - b'1'
        } else {
            return Err("invalid row");
        };
        Ok(Self { row, column })
    }
}

impl ChessBoardPosition {
    /// return a distance to the rhs position which must be reachable by Queen allowed moves
    ///
    /// ```
    /// use std::str::FromStr;
    /// use chess_interactor::ChessBoardPosition;
    /// let pos = ChessBoardPosition::from_str("a1").unwrap();
    /// assert_eq!(pos.queen_distance(&ChessBoardPosition::from_str("a1").unwrap()), Ok((0, (0, 0))));
    /// assert_eq!(pos.queen_distance(&ChessBoardPosition::from_str("a2").unwrap()), Ok((1, (1, 0))));
    /// assert_eq!(pos.queen_distance(&ChessBoardPosition::from_str("b1").unwrap()), Ok((1, (0, 1))));
    /// assert_eq!(pos.queen_distance(&ChessBoardPosition::from_str("b2").unwrap()), Ok((1, (1, 1))));
    /// assert_eq!(pos.queen_distance(&ChessBoardPosition::from_str("h8").unwrap()), Ok((7, (1, 1))));
    /// assert!(pos.queen_distance(&ChessBoardPosition::from_str("b3").unwrap()).is_err());
    /// assert!(pos.queen_distance(&ChessBoardPosition::from_str("b4").unwrap()).is_err());
    /// assert!(pos.queen_distance(&ChessBoardPosition::from_str("c2").unwrap()).is_err());
    /// let pos = ChessBoardPosition::from_str("d4").unwrap();
    /// assert_eq!(pos.queen_distance(&ChessBoardPosition::from_str("a1").unwrap()), Ok((3, (-1, -1))));
    /// ```
    pub fn queen_distance(&self, rhs: &Self) -> Result<(u8, (i8, i8)), &'static str> {
        let row_diff = i16::from(rhs.row) - i16::from(self.row);
        let column_diff = i16::from(rhs.column) - i16::from(self.column);
        if row_diff == 0 && column_diff == 0 {
            return Ok((0, (0, 0)));
        }
        let distance = if row_diff == 0 || column_diff == 0 {
            (row_diff.abs() + column_diff.abs()) as u8
        } else if row_diff.abs() == column_diff.abs() {
            ((row_diff.abs() + column_diff.abs()) / 2) as u8
        } else {
            return Err("the position is not reachable by queen move");
        };

        Ok((
            distance,
            (
                (row_diff / i16::from(distance)) as i8,
                (column_diff / i16::from(distance)) as i8,
            ),
        ))
    }
}

#[derive(Debug)]
pub enum ChessPiece {
    King,
    Queen,
}

impl std::str::FromStr for ChessPiece {
    type Err = &'static str;

    /// ```
    /// use std::str::FromStr;
    /// use chess_interactor::ChessPiece;
    /// assert!(matches!(ChessPiece::from_str("K"), Ok(ChessPiece::King)));
    /// assert!(matches!(ChessPiece::from_str("Q"), Ok(ChessPiece::Queen)));
    /// assert!(matches!(ChessPiece::from_str(" "), Err("invalid chess piece")));
    /// assert!(matches!(ChessPiece::from_str(""), Err("invalid chess piece")));
    /// assert!(matches!(ChessPiece::from_str("X"), Err("invalid chess piece")));
    /// assert!(matches!(ChessPiece::from_str("         "), Err("invalid chess piece")));
    /// assert!(matches!(ChessPiece::from_str("1"), Err("invalid chess piece")));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "K" => Ok(Self::King),
            "Q" => Ok(Self::Queen),
            _ => Err("invalid chess piece"),
        }
    }
}

pub struct Chess {
    white_king_position: ChessBoardPosition,
    white_queen_position: ChessBoardPosition,
    black_king_position: ChessBoardPosition,
    moves: u64,
    moves_limit: u64,
}

#[derive(Debug)]
pub enum GameOver {
    WrongInput {
        error_message: String,
        input: String,
    },
    TooManyMoves,
    Draw,
    Stalemate,
    Checkmate,
}

impl Chess {
    pub fn new(
        white_king_position: ChessBoardPosition,
        white_queen_position: ChessBoardPosition,
        black_kind_position: ChessBoardPosition,
        moves_limit: u64,
    ) -> Self {
        Self {
            white_king_position,
            white_queen_position,
            black_king_position: black_kind_position,
            moves: 0,
            moves_limit,
        }
    }

    pub fn moves(&self) -> u64 {
        self.moves
    }

    pub fn play(&mut self) -> GameOver {
        let mut line = String::new();
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        println!(
            "{} {} {}",
            self.white_king_position, self.white_queen_position, self.black_king_position
        );
        info!(target: "game_log", "{} {} {}", self.white_king_position, self.white_queen_position, self.black_king_position);
        loop {
            if self.moves >= self.moves_limit {
                return GameOver::TooManyMoves;
            }

            line.clear();
            if let Err(error) = stdin.read_line(&mut line) {
                return GameOver::WrongInput {
                    error_message: format!(
                        "Reading a new line from a solution failed: {:?}",
                        error
                    ),
                    input: "".into(),
                };
            }
            let line = line.trim();
            info!(target: "game_log", "{}", line);

            let checkmate = if line.len() == 4 && line.ends_with("#") {
                true
            } else if line.len() == 3 {
                false
            } else {
                return GameOver::WrongInput {
                    error_message: "line is neither of length 3 nor length 4 with '#' at the end"
                        .into(),
                    input: line.into(),
                };
            };

            let chess_piece: ChessPiece = match line[..1].parse() {
                Ok(chess_piece) => chess_piece,
                Err(err) => {
                    return GameOver::WrongInput {
                        error_message: err.into(),
                        input: line.into(),
                    };
                }
            };

            let chess_piece_move: ChessBoardPosition = match line[1..3].parse() {
                Ok(chess_piece_move) => chess_piece_move,
                Err(err) => {
                    return GameOver::WrongInput {
                        error_message: err.into(),
                        input: line.into(),
                    };
                }
            };

            if let Err(err) = self.try_apply_move(chess_piece, chess_piece_move) {
                return GameOver::WrongInput {
                    error_message: err.into(),
                    input: line.into(),
                };
            }
            self.moves += 1;

            if self
                .black_king_position
                .queen_distance(&self.white_queen_position)
                .map(|(distance, _)| distance)
                .unwrap_or(0)
                == 1
                && self
                    .white_king_position
                    .queen_distance(&self.white_queen_position)
                    .map(|(distance, _)| distance)
                    .unwrap_or(0)
                    != 1
            {
                debug!(
                    "White queen moved too close to the black king without white king protection"
                );
                return GameOver::Draw;
            }

            if let Err(game_over) = self.try_move_black_king() {
                if let GameOver::Checkmate = game_over {
                    if !checkmate {
                        return GameOver::WrongInput {
                            error_message: "no checkmate when expected".into(),
                            input: line.into(),
                        };
                    }
                }
                return game_over;
            };

            println!("K{}", self.black_king_position);
            info!(target: "game_log", "K{}", self.black_king_position);
        }
    }

    fn try_apply_move(
        &mut self,
        chess_piece: ChessPiece,
        chess_piece_move: ChessBoardPosition,
    ) -> Result<(), &'static str> {
        match chess_piece {
            #[cfg(not(feature = "king-moves-enabled"))]
            ChessPiece::King => return Err("king moves are not allowed"),
            #[cfg(feature = "king-moves-enabled")]
            ChessPiece::King => {
                debug!(
                    "Tring to move white king from {} to {}",
                    self.white_king_position, chess_piece_move
                );
                if self.white_queen_position == chess_piece_move {
                    return Err("king tried to move over the queen");
                }
                let (distance, _) = self
                    .white_king_position
                    .queen_distance(&chess_piece_move)
                    .map_err(|_| "king tried to do impossible move")?;
                if distance == 0 {
                    return Err("king was not moved");
                }
                if distance > 1 {
                    return Err("king tried to move too far");
                }
                if let Ok((1, _)) = self.black_king_position.queen_distance(&chess_piece_move) {
                    return Err("white king tried to move next to the black king");
                }
                self.white_king_position = chess_piece_move;
            }
            ChessPiece::Queen => {
                debug!(
                    "Tring to move white queen from {} to {}",
                    self.white_queen_position, chess_piece_move
                );
                let (distance_to_new_position, direction_to_new_position) = self
                    .white_queen_position
                    .queen_distance(&chess_piece_move)
                    .map_err(|_| "queen tried to do impossible move")?;
                if distance_to_new_position == 0 {
                    return Err("queen has not been moved");
                }

                if let Ok((distance_to_white_king, direction_to_white_king)) = self
                    .white_queen_position
                    .queen_distance(&self.white_king_position)
                {
                    if direction_to_new_position == direction_to_white_king
                        && distance_to_new_position >= distance_to_white_king
                    {
                        return Err("queen tried to jump over white king");
                    }
                }

                if let Ok((distance_to_black_king, direction_to_black_king)) = self
                    .white_queen_position
                    .queen_distance(&self.black_king_position)
                {
                    if direction_to_new_position == direction_to_black_king
                        && distance_to_new_position >= distance_to_black_king
                    {
                        return Err("queen tried to jump over black king");
                    }
                }

                self.white_queen_position = chess_piece_move;
            }
        }
        Ok(())
    }

    fn try_move_black_king(&mut self) -> Result<(), GameOver> {
        #[derive(Debug, Clone, Copy)]
        enum ChessBoardCell {
            Available,
            King,
            Attackable,
        }

        let mut board = [[ChessBoardCell::Available; 8]; 8];

        // Mark attackable cells by white king
        for row in usize::from(self.white_king_position.row.saturating_sub(1))
            ..=usize::from(self.white_king_position.row + 1).min(7)
        {
            for column in usize::from(self.white_king_position.column.saturating_sub(1))
                ..=usize::from(self.white_king_position.column + 1).min(7)
            {
                board[row][column] = ChessBoardCell::Attackable;
            }
        }
        board[usize::from(self.white_king_position.row)]
            [usize::from(self.white_king_position.column)] = ChessBoardCell::King;

        // Mark attackable cells by white queen to the right
        let row = usize::from(self.white_queen_position.row);
        for column in (usize::from(self.white_queen_position.column) + 1)..=7 {
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }
        // Mark attackable cells by white queen to the left
        for column in (0..usize::from(self.white_queen_position.column)).rev() {
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }

        // Mark attackable cells by white queen up
        let column = usize::from(self.white_queen_position.column);
        for row in (usize::from(self.white_queen_position.row) + 1)..=7 {
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }
        // Mark attackable cells by white queen down
        for row in (0..usize::from(self.white_queen_position.row)).rev() {
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }

        // Mark attackable cells by white queen up right
        let mut column = usize::from(self.white_queen_position.column);
        let mut row = usize::from(self.white_queen_position.row);
        while row < 7 && column < 7 {
            column += 1;
            row += 1;
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }

        // Mark attackable cells by white queen down left
        let mut column = usize::from(self.white_queen_position.column);
        let mut row = usize::from(self.white_queen_position.row);
        while row > 0 && column > 0 {
            column -= 1;
            row -= 1;
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }

        // Mark attackable cells by white queen down right
        let mut column = usize::from(self.white_queen_position.column);
        let mut row = usize::from(self.white_queen_position.row);
        while row > 0 && column < 7 {
            column += 1;
            row -= 1;
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }

        // Mark attackable cells by white queen up left
        let mut column = usize::from(self.white_queen_position.column);
        let mut row = usize::from(self.white_queen_position.row);
        while row < 7 && column > 0 {
            column -= 1;
            row += 1;
            if let ChessBoardCell::King = board[row][column] {
                break;
            }
            board[row][column] = ChessBoardCell::Attackable;
        }

        let mut best_new_position = self.black_king_position.clone();
        let mut best_new_position_space = 0;

        for row in self.black_king_position.row.saturating_sub(1)
            ..=(self.black_king_position.row + 1).min(7)
        {
            for column in self.black_king_position.column.saturating_sub(1)
                ..=(self.black_king_position.column + 1).min(7)
            {
                if row == self.black_king_position.row && column == self.black_king_position.column
                {
                    continue;
                }
                // TODO: implement a proper strategy!
                if let ChessBoardCell::Available = board[usize::from(row)][usize::from(column)] {
                    best_new_position = ChessBoardPosition { row, column };
                    best_new_position_space = 1;
                }
            }
        }

        if best_new_position_space == 0 {
            if let ChessBoardCell::Available = board[usize::from(self.black_king_position.row)]
                [usize::from(self.black_king_position.column)]
            {
                return Err(GameOver::Stalemate);
            }
            return Err(GameOver::Checkmate);
        }

        self.black_king_position = best_new_position;

        Ok(())
    }
}
