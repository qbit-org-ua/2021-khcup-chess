use std::str::FromStr;

use tracing::info;

use chess_interactor::{Chess, ChessBoardPosition, GameOver};

const EXIT_CODE_OK: i32 = 0;
const EXIT_CODE_WA: i32 = 1;
const EXIT_CODE_PE: i32 = 2;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Initializing Chess interactor");

    let game_initial_state =
        std::fs::read_to_string("answer.txt").expect("unable to read answer.txt");
    let mut game_initial_state = game_initial_state.split_ascii_whitespace().map(|position| {
        ChessBoardPosition::from_str(position)
            .expect("unable to parse initial chess piece positions")
    });
    let white_king_position = game_initial_state
        .next()
        .expect("unable to find the initial white king position");
    let white_queen_position = game_initial_state
        .next()
        .expect("unable to find the initial white queen position");
    let black_king_position = game_initial_state
        .next()
        .expect("unable to find the initial black king position");

    let mut chess = Chess::new(
        white_king_position,
        white_queen_position,
        black_king_position,
        50,
    );

    let game_status = chess.play();
    info!("{:?}. Moves: {}", game_status, chess.moves());

    let exit_code = match game_status {
        GameOver::Checkmate => EXIT_CODE_OK,
        GameOver::WrongInput { .. } => EXIT_CODE_PE,
        GameOver::TooManyMoves | GameOver::Draw | GameOver::Stalemate => EXIT_CODE_WA,
    };
    std::process::exit(exit_code);
}
