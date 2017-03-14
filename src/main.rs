extern crate rand;

use std::io;
use std::io::Read;
use rand::Rng;
use std::collections::vec_deque::VecDeque;

#[derive(Copy)]
#[derive(PartialEq)]
enum FieldState {
    NotFlagged,
    Flagged,
    Revealed,
}

impl Clone for FieldState {
    fn clone(&self) -> FieldState { *self }
}

#[derive(Copy)]
struct Field {
    has_bomb: bool,
    state: FieldState,
}

impl Clone for Field {
    fn clone(&self) -> Field { *self }
}

const ADJACENT: [(i32, i32); 8] =
    [(-1, -1), (-1,  0), (-1,  1),
     ( 0, -1),           ( 0,  1),
     ( 1, -1), ( 1,  0), ( 1,  1)];

struct Board {
    x: usize,
    y: usize,

    cursor_x: usize,
    cursor_y: usize,

    left_to_reveal: i32,

    fields: Vec<Vec<Field>>,
    mine_counter: Vec<Vec<i32>>,
}

#[derive(PartialEq)]
enum GameResult {
    Win,
    Loss,
    Unknown,
}

impl Board {
    fn new(x: usize, y: usize, bombs: u32) -> Board {
        if ((x * y) as u32) < bombs {
            panic!("Too many bombs!");
        }

        let mut fields = vec![vec![Field{ has_bomb: false, state: FieldState::NotFlagged }; x]; y];
        let mut mine_counter = vec![vec![0; x]; y];

        for _ in 0..bombs {
            loop {
                let x_r = rand::thread_rng().gen_range(0, x);
                let y_r = rand::thread_rng().gen_range(0, y);

                if !fields[y_r][x_r].has_bomb {
                    fields[y_r][x_r].has_bomb = true;
                    break;
                }
            }
        }

        let is_valid = |xx: i32, yy: i32| -> bool {
            xx >= 0 && xx < x as i32 && yy >= 0 && yy < y as i32
        };

        for i in 0..x {
            for j in 0..y {
                for &(ii, jj) in ADJACENT.iter() {
                    let xx = (i as i32 + ii) as i32;
                    let yy = (j as i32 + jj) as i32;

                    if !is_valid(xx, yy) { continue; }

                    if fields[yy as usize][xx as usize].has_bomb {
                        mine_counter[j][i] += 1;
                    }
                }
            }
        }

        return Board {
            x: x,
            y: y,
            cursor_x: x / 2,
            cursor_y: y / 2,
            left_to_reveal: (x * y) as i32 - bombs as i32,
            fields: fields,
            mine_counter: mine_counter
        };
    }

    fn display(&self) {
        let num_to_char = |num: i32| {
            (num as u8 + '0' as u8) as char
        };

        for i in 0..self.y {
            for j in 0..self.x {
                let field = &self.fields[i][j];

                let character: char = match field.state {
                    FieldState::NotFlagged => '.',
                    FieldState::Flagged    => '?',
                    FieldState::Revealed   =>
                        if !field.has_bomb { num_to_char(self.mine_counter[i][j]) } else { 'X' },
                };

                if i == self.cursor_y && j == self.cursor_x {
                    print!("[{}]", character);
                } else {
                    print!(" {} ", character);
                }
            }
            println!();
        }
    }

    fn update(&mut self, input: char) -> GameResult {
        match input {
            '8' => self.cursor_y -= if self.cursor_y > 0 {1} else {0},
            '5' => self.cursor_y += if self.cursor_y < self.y - 1 {1} else {0},
            '4' => self.cursor_x -= if self.cursor_x > 0 {1} else {0},
            '6' => self.cursor_x += if self.cursor_x < self.x - 1 {1} else {0},

            '7' => {
                let (x, y) = (self.cursor_x, self.cursor_y);
                return self.reveal(x, y);
            },
            '9' => self.mark(),

            _   => { }
        }

        GameResult::Unknown
    }

    fn reveal(&mut self, x: usize, y: usize) -> GameResult {
        if self.fields[y][x].has_bomb {
            self.fields[y][x].state = FieldState::Revealed;
            return GameResult::Loss;
        }

        let mut queue = VecDeque::<(usize, usize)>::new();
        queue.push_back((x, y));

        while !queue.is_empty() {
            let (x, y) = match queue.pop_front() {
                Some((xx, yy)) => (xx, yy),
                None           => continue,
            };
            let field = &mut self.fields[y][x];

            if field.state != FieldState::Revealed {
                field.state = FieldState::Revealed;
                self.left_to_reveal -= 1;

                if self.mine_counter[y][x] == 0 {
                    for &(xa, ya) in ADJACENT.iter() {
                        let xx = x as i32 + xa;
                        let yy = y as i32 + ya;

                        if xx < 0 || xx >= self.x as i32 || yy < 0 || yy >= self.y as i32 {
                            continue;
                        }

                        queue.push_back((xx as usize, yy as usize));
                    }
                }
            }
        }

        if self.left_to_reveal == 0 {
            GameResult::Win
        } else {
            GameResult::Unknown
        }
    }

    fn mark(&mut self) {
        let (x, y) = (self.cursor_x, self.cursor_y);
        if self.fields[y][x].state == FieldState::NotFlagged {
            self.fields[y][x].state = FieldState::Flagged;
        }
    }
}

fn get_char() -> char {
    let mut character = [0];
    io::stdin().read(&mut character)
        .expect("Failed to read input");
    return character[0] as char;
}

fn clear_screen() {
    print!("{}[2J", 27 as char);
}

fn main() {
    //  10x10 board with 10 bombs
    let mut board = Board::new(10, 10, 10);

    loop {
        clear_screen();
        board.display();

        println!();
        println!("[8] - up");
        println!("[5] - down");
        println!("[4] - left");
        println!("[6] - right");
        println!("[7] - reveal");
        println!("[9] - mark");
        println!("[q] - quit");

        let input = get_char();
        if input == 'q' {
            break;
        }

        let result = board.update(input);
        if result != GameResult::Unknown {
            clear_screen();
            board.display();

            println!("{}", match result {
                GameResult::Win  => "You win!",
                GameResult::Loss => "You lose!",
                _                => "",
            });
            break;
        }
    }
}
