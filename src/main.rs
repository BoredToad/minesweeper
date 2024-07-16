extern crate ncurses as nc;
use nc::{
    COLOR_BLACK, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_RED, COLOR_WHITE,
    COLOR_YELLOW,
};
use rand::Rng;

#[derive(Clone)]
enum Heat {
    Bomb,
    None,
    Neighbours(u8),
}

#[derive(Clone)]
struct Square {
    revealed: bool,
    heat: Heat,
}

type Board = Vec<Vec<Square>>;

struct Game {
    // board[height(row)][width(column)]
    board: Board,
    cursor: (usize, usize),
    bomb_count: u32,
    bombs_cleared: u32,
}

impl Square {
    pub fn new(percentage: u8, count: &mut u32) -> Square {
        Square {
            // change to false later
            revealed: true,
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
        for i in y - if y == 0 { 0 } else { 1 }..=y + 1 {
            if i >= board.len() {
                continue;
            }
            for j in x - if x == 0 { 0 } else { 1 }..=x + 1 {
                if j >= board[0].len() {
                    continue;
                }
                if let Heat::Bomb = board[i][j].heat {
                    count += 1
                }
            }
        }

        if count == 0 {
            return Heat::None;
        }

        Heat::Neighbours(count)
    }

    pub fn clear(self, x: Game) {}
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
                    revealed: square.revealed,
                    heat: square.get_heat((i, j), &board),
                })
                .collect()
        })
        .collect()
}

impl Game {
    // dimensions (height, width)
    pub fn init(dimensions: (usize, usize), percentage: u8) -> Game {
        nc::initscr();
        nc::raw();
        // honestly don't like this approach to colors
        nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE); //   |
        nc::init_pair(1, COLOR_BLUE, COLOR_WHITE);
        nc::init_pair(2, COLOR_GREEN, COLOR_WHITE);
        nc::init_pair(3, COLOR_RED, COLOR_WHITE);
        nc::init_pair(4, COLOR_CYAN, COLOR_WHITE);
        nc::init_pair(5, COLOR_YELLOW, COLOR_WHITE);
        nc::init_pair(6, COLOR_MAGENTA, COLOR_WHITE);
        nc::init_pair(7, COLOR_BLACK, COLOR_WHITE);
        nc::init_pair(8, COLOR_BLUE, COLOR_WHITE); // sadly reused color

        let (board, count) = board_with_bombs(dimensions, percentage);
        Game {
            board: get_neighbours(board),
            cursor: (0, 0),
            bomb_count: count,
            bombs_cleared: 0,
        }
    }

    fn render(&self) {
        nc::clear();

        for row in self.board.iter() {
            for square in row.iter() {
                nc::addch(match square.heat {
                    Heat::Bomb => 'X',
                    Heat::None => ' ',
                    Heat::Neighbours(i) => {
                        nc::attron(nc::COLOR_PAIR(i as i16));
                        char::from_u32(i as u32 + 48).expect("RESULT")
                    }
                } as u32);
            }
            nc::addch('\n' as u32);
        }

        nc::refresh();
        // nc::addstr(&self.debug_string());
    }

    fn handle_input(&mut self, input: i32) {
        match char::from_u32(input as u32).expect("REASON") {
            'q' => {
                nc::endwin();
                panic!("Force quit!")
            }
            _ => {}
        }
    }

    pub fn run(mut self) {
        // println!("{}", self.debug_string());
        while self.bombs_cleared < self.bomb_count {
            self.render();
            self.handle_input(nc::getch());

            // break; // tmp
        }
        nc::endwin();
    }

    fn debug_string(&self) -> String {
        let mut out = String::new();
        for row in self.board.iter() {
            for square in row.iter() {
                out += " | ";
                out.push(match square.heat {
                    Heat::Bomb => 'X',
                    Heat::None => ' ',
                    Heat::Neighbours(i) => char::from_digit(i.into(), 10).expect("REASON"),
                })
            }
            out += " |\n";
        }
        out
    }
}

fn main() {
    // Plan of attack:
    // print empty board once
    // initial selection
    // gen board:
    //      loop over every square and make it Bomb or Empty(0)
    //          avoid the selected squares
    //      loop over the squares again to assign neighbours
    //  game loop:
    //      print
    //      wait for input,
    //      clear square( do recursively if 0 until number ) or die
    //  print ending screen
    // curses_test()

    Game::init((5, 5), 25).run();
}

fn curses_test() {
    // here I have all the functions I need!

    // init
    nc::initscr();
    nc::raw();
    nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    // prints str
    nc::addstr("Input: ");

    // gets ch
    let ch = nc::getch();

    // example of printing input
    if ch == ' ' as i32 {
        // attron and attroff are for cool effects
        nc::attron(nc::A_BOLD() | nc::A_BLINK());
        nc::addstr("<SPACE>");
        nc::attroff(nc::A_BOLD() | nc::A_BLINK());
        nc::addstr(" pressed");
    } else {
        nc::addstr("\nKey pressed: ");
        nc::attron(nc::A_BOLD() | nc::A_BLINK());
        nc::addstr(format!("{}\n", char::from_u32(ch as u32).expect("Invalid char")).as_ref());
        nc::attroff(nc::A_BOLD() | nc::A_BLINK());
    }

    // to actually print
    nc::refresh();

    nc::getch();

    // clear, duh
    nc::clear();

    nc::addstr("ok...");
    nc::refresh();

    nc::getch();
    // to end shidd!
    nc::endwin();
}
