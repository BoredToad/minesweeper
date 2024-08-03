extern crate ncurses as nc;

use nc::{
    COLOR_BLACK, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_RED, COLOR_WHITE,
    COLOR_YELLOW,
};
use rand::Rng;

enum Heat {
    Bomb,
    None,
    Neighbours(u8),
}

struct Square {
    revealed: bool,
    marked: bool,
    heat: Heat,
}

type Board = Vec<Vec<Square>>;

struct Game {
    board: Board,
    cursor: (usize, usize),
    bomb_count: u32,
    squares_revealed: u32,
}

enum GameExit {
    Loss,
    Quit,
    Resume,
}

impl Square {
    pub fn new(percentage: u8, count: &mut u32) -> Square {
        Square {
            // change to false later
            revealed: false,
            marked: false,
            heat: if rand::thread_rng().gen_range(1..=100) <= percentage {
                *count += 1;
                Heat::Bomb
            } else {
                Heat::None
            },
        }
    }

    pub fn get_heat(&self, (y, x): (usize, usize), board: &Board) -> Heat {
        if let Heat::Bomb = self.heat {
            return Heat::Bomb;
        }

        let mut count = 0u8;
        // UGLY ASS CODE,
        // plz fix later
        for i in y as isize - 1..=y as isize + 1 {
            if i < 0 || i >= board.len() as isize {
                continue;
            }
            for j in x as isize - 1..=x as isize + 1 {
                if j < 0 || j >= board[0].len() as isize {
                    continue;
                }
                if let Heat::Bomb = board[i as usize][j as usize].heat {
                    count += 1
                }
            }
        }

        if count == 0 {
            return Heat::None;
        }

        Heat::Neighbours(count)
    }

    pub fn reveal((y, x): (usize, usize), board: &mut Board) -> i32 {
        let square = &mut board[y][x];
        square.revealed = true;

        if let Heat::Bomb = square.heat {
            return -1;
        }

        let mut revealed = 1i32;
        if let Heat::Neighbours(_) = square.heat {
            return revealed;
        }

        for i in y as isize - 1..=y as isize + 1 {
            if i < 0 || i >= board.len() as isize {
                continue;
            }
            for j in x as isize - 1..=x as isize + 1 {
                if j < 0 || j >= board[0].len() as isize {
                    continue;
                }
                if !board[i as usize][j as usize].revealed {
                    revealed += Square::reveal((i as usize, j as usize), board);
                }
            }
        }
        revealed
    }
}

fn board_with_bombs((height, width): (usize, usize), percentage: u8) -> (Board, u32) {
    let mut count = 0;
    let board = (0..height)
        .map(|_| {
            (0..width)
                .map(|_| Square::new(percentage, &mut count))
                .collect()
        })
        .collect();
    (board, count)
}

fn get_neighbours(board: Board) -> Board {
    board
        .iter()
        .enumerate()
        .map(|(i, row)| {
            row.iter()
                .enumerate()
                .map(|(j, square)| Square {
                    heat: square.get_heat((i, j), &board),
                    ..*square
                })
                .collect()
        })
        .collect()
}

impl Game {
    // dimensions (height, width)
    pub fn init(dimensions: (usize, usize), percentage: u8) -> Game {
        Game::nc_init();
        let (board, count) = board_with_bombs(dimensions, percentage);
        Game {
            board: get_neighbours(board),
            cursor: (0, 0),
            bomb_count: count,
            squares_revealed: 0,
        }
    }

    fn nc_init() {
        nc::initscr();
        nc::raw();
        nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        nc::start_color();
        // basic pairs for revealed colors
        nc::init_pair(11, COLOR_WHITE, COLOR_BLACK);
        nc::init_pair(1, COLOR_BLUE, COLOR_BLACK);
        nc::init_pair(2, COLOR_GREEN, COLOR_BLACK);
        nc::init_pair(3, COLOR_RED, COLOR_BLACK);
        nc::init_pair(4, COLOR_YELLOW, COLOR_BLACK);
        nc::init_pair(5, COLOR_MAGENTA, COLOR_BLACK);
        nc::init_pair(6, COLOR_CYAN, COLOR_BLACK);
        nc::init_pair(7, COLOR_BLUE, COLOR_BLACK);
        nc::init_pair(8, COLOR_RED, COLOR_BLACK);

        nc::init_pair(9, COLOR_BLACK, COLOR_RED); // marked
        nc::init_pair(10, COLOR_BLACK, COLOR_WHITE); // hidden
    }

    pub fn run(mut self) {
        let mut exit_message = String::from("You win!");
        while self.squares_revealed
            < (self.board.len() * self.board[0].len()) as u32 - self.bomb_count
        {
            self.render();

            exit_message = match self.handle_input(nc::getch()) {
                GameExit::Loss => String::from("You lost!"),
                GameExit::Quit => String::from("You quit!"),
                GameExit::Resume => continue,
            };
            break;
        }

        // nc::endwin();
        self.render();
        println!("{}", exit_message);
        nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_VISIBLE); // needed to restore to default
    }

    fn render(&self) {
        nc::clear();

        for (i, row) in self.board.iter().enumerate() {
            for (j, square) in row.iter().enumerate() {
                self.draw_char((i, j), &square);
            }
            nc::addch('\n' as u32);
        }
        // nc::addstr(
        //     format!(
        //         "({}, {})\n CLEARED: {}, REMAINING: {}",
        //         self.cursor.0,
        //         self.cursor.1,
        //         self.squares_revealed,
        //         (self.board.len() * self.board[0].len()) as u32
        //             - self.bomb_count
        //             - self.squares_revealed
        //     )
        //     .as_str(),
        // ); // for debugging

        nc::refresh();
    }

    fn draw_char(&self, (i, j): (usize, usize), square: &Square) {
        nc::attron(nc::COLOR_PAIR(10));
        if (i, j) == self.cursor {
            nc::attron(nc::A_STANDOUT());
        }

        nc::addch(if square.revealed {
            match square.heat {
                Heat::Bomb => {
                    nc::attron(nc::COLOR_PAIR(9));
                    nc::attron(nc::A_BLINK());
                    nc::attron(nc::A_BOLD());
                    'X'
                }
                Heat::None => {
                    nc::attron(nc::COLOR_PAIR(11));
                    ' '
                }
                Heat::Neighbours(i) => {
                    nc::attron(nc::COLOR_PAIR(i as i16));
                    char::from_u32(i as u32 + 48).expect("RESULT")
                }
            }
        } else if square.marked {
            nc::attron(nc::COLOR_PAIR(9));
            '?'
        } else {
            ' '
        } as u32);

        nc::standend();
    }

    fn handle_input(&mut self, input: i32) -> GameExit {
        match char::from_u32(input as u32).expect("REASON") {
            'q' => {
                nc::endwin();
                return GameExit::Quit;
            }
            ' ' | '\n' => {
                let revealed = Square::reveal(self.cursor, &mut self.board);
                if revealed == -1 {
                    return GameExit::Loss;
                }
                self.squares_revealed += revealed as u32;
            }
            'm' | 'f' | '\t' => {
                self.board[self.cursor.0][self.cursor.1].marked =
                    !self.board[self.cursor.0][self.cursor.1].marked
            }
            other => self.move_cursor(match other {
                'h' | 'a' => (0, -1),
                'l' | 'd' => (0, 1),
                'k' | 'w' => (-1, 0),
                'j' | 's' => (1, 0),
                _ => return GameExit::Resume,
            }),
        };
        GameExit::Resume
    }

    fn move_cursor(&mut self, input: (i32, i32)) {
        let new = (
            input.0 + self.cursor.0 as i32,
            input.1 + self.cursor.1 as i32,
        );

        if !((new.0 < 0 || new.0 > self.board.len() as i32 - 1)
            || (new.1 < 0 || new.1 > self.board[0].len() as i32 - 1))
        {
            self.cursor = (new.0 as usize, new.1 as usize)
        }
    }
}

#[derive(Debug)]
enum ArgError {
    TooFewArgs,
    TooManyArgs,
    BadBoardSize(String),
    BadDifficulty(String),
}

fn main() -> Result<(), ArgError> {
    let (board_size, bomb_frequency) = handle_args()?;
    Game::init(board_size, bomb_frequency).run();
    Ok(())
}

fn handle_args() -> Result<((usize, usize), u8), ArgError> {
    let mut board_size = (5, 5);
    let mut bomb_frequency = 15;
    if std::env::args().count() < 3 {
        println!("\nToo few args! Usage:\n\tminesweeper [board_size ([S]mall, [M]edium, [L]arge)] [difficulty ([E]asy, [N]ormal, [H]ard)]");
        return Err(ArgError::TooFewArgs);
    }
    for (i, arg) in std::env::args().enumerate() {
        match i {
            0 => {}
            1 => {
                board_size = match arg.to_lowercase().as_str() {
                    "small" | "s" => (5, 10),
                    "medium" | "m" => (10, 20),
                    "large" | "l" => (20, 40),
                    _ => {
                        return Err(ArgError::BadBoardSize(String::from(
                            "opts: [S]mall, [M]edium, [L]arge",
                        )))
                    }
                }
            }
            2 => {
                bomb_frequency = match arg.to_lowercase().as_str() {
                    "easy" | "e" => 10,
                    "normal" | "n" => 15,
                    "hard" | "h" => 20,
                    _ => {
                        return Err(ArgError::BadDifficulty(String::from(
                            "opts: [E]asy, [N]ormal, [H]ard",
                        )))
                    }
                }
            }
            _ => {
                return Err(ArgError::TooManyArgs);
            }
        }
    }
    Ok((board_size, bomb_frequency))
}
