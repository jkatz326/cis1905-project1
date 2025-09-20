use std::error::Error;
use std::fmt::Display;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameStatus {
    Win,
    Lose,
    Continue,
}

//Error Handling 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardError {
    InvalidCharacter(char),
    InvalidSize,
    NoMinotaur,
    NoTheseus,
    NoGoal,
    MultipleMinotaur,
    MultipleTheseus,
    MultipleGoal,
}
impl Display for BoardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardError::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            BoardError::InvalidSize => write!(f, "Invalid size"),
            BoardError::NoMinotaur => write!(f, "No minotaur"),
            BoardError::NoTheseus => write!(f, "No theseus"),
            BoardError::NoGoal => write!(f, "No goal"),
            BoardError::MultipleMinotaur => write!(f, "Multiple minotaur"),
            BoardError::MultipleTheseus => write!(f, "Multiple theseus"),
            BoardError::MultipleGoal => write!(f, "Multiple goal"),
        }
    }
}
impl Error for BoardError {}

//Helper for grid.from_board that processes a char from board input string
pub fn process_char(chr: char, curr_pos: (usize, usize), g_pos: &mut Option<(usize, usize)>, t_pos: &mut Option<(usize, usize)>, m_pos: &mut Option<(usize, usize)>) -> Result<char, BoardError>{
    if chr == ' ' || chr == 'X' {
        //Just return for empty and walls
        return Ok(chr)
    } else if chr == 'G' {
        //Save this location as goal if first found, o/w error
        match *g_pos {
            Some(_) => return Err(BoardError::MultipleGoal),
            None => {
                *g_pos = Some(curr_pos);
                return Ok(chr);
            }
        }
    } else if chr == 'T' {
        //Save this location as theseus if first found, o/w error
        match *t_pos {
            Some(_) => return Err(BoardError::MultipleTheseus),
            None => {
                *t_pos = Some(curr_pos);
                return Ok(chr);
            }
        }
    } else if chr == 'M' {
        //Save this location as minotaur if first found, o/w error
        match *m_pos {
            Some(_) => return Err(BoardError::MultipleMinotaur),
            None => {
                *m_pos = Some(curr_pos);
                return Ok(chr);
            }
        }
    } 
    //All other characters are invalid
    return Err(BoardError::InvalidCharacter(chr));
}

#[derive(Clone)]
pub struct Grid {
    vec: Vec<Vec<char>>, //Each inner vector is a row of characters, vec of vecs make a grid, index by [row][col]
    t_pos: (usize, usize), //theseus position (row index, col index)
    m_pos: (usize, usize), //minotaur position (row index, col index)
    row_bound: usize, //highest index for any row in grid
    col_bound: usize, //highest index for any column in grid
}

impl Grid {
    //Takes in string and constructs a Grid struct
    pub fn from_board(board: &str) -> Result<Grid, BoardError> {
        let mut g_pos: Option<(usize, usize)> = None;
        let mut t_pos: Option<(usize, usize)> = None;
        let mut m_pos: Option<(usize, usize)> = None;
        let mut col_bound: usize = 0;
        let mut grid: Vec<Vec<char>> = Vec::new();
        //use helper to process each char in grid, throw errors as needed
        for (row, line) in board.lines().enumerate() {
            let mut row_vec: Vec<char> = Vec::new();
            for (col, c) in line.chars().enumerate() {
                match process_char(c, (row, col), &mut g_pos, &mut t_pos, &mut m_pos) {
                    Ok(_) => {
                        row_vec.push(c);
                    },
                    Err(err) => return Err(err),
                }
            }
            col_bound = col_bound.max(row_vec.len() - 1); //must check each row to see which is longest, max is column bound
            grid.push(row_vec); 
        }
        let row_bound = grid.len() - 1;
        //Throw errors if goal, theseus or minotaur not found
        if g_pos.is_none() {
            return Err(BoardError::NoGoal)
        } else if t_pos.is_none() {
            return Err(BoardError::NoTheseus)
        } else if m_pos.is_none() {
            return Err(BoardError::NoMinotaur)
        } else {
            return Ok(Grid {vec: grid, t_pos: t_pos.unwrap(), m_pos: m_pos.unwrap(), row_bound: row_bound, col_bound: col_bound})
        }
    }

    //checks outer bounds of grid
    pub fn in_bounds(&self, pos: (usize, usize)) -> bool {
        return pos.0 <= self.row_bound && pos.1 <= self.col_bound;
    }
}

#[derive(Clone)]
pub struct Game {
    grid: Grid, //contains most information about board, see above
    status: GameStatus, //win, lose, or continue
}

impl Game {
    // wrapper for Grid.from_board
    pub fn from_board(board: &str) -> Result<Game, BoardError> {
        let grid : Grid;
        match Grid::from_board(board) {
            Ok(g) => grid = g,
            Err(err) => return Err(err),
        }
        return Ok(Game {grid: grid, status: GameStatus::Continue})
    }

    // display grid
    pub fn show(&self) {
        for row in self.grid.vec.iter() {
            for chr in row.iter() {
                print!("{}", chr);
            }
            println!(" ");
        }
    }

    // moves the minotaur one space according to standard algo if possible (no moving through goal)
    pub fn minotaur_move(&mut self) {
        let new_pos: (usize, usize);
        //If minotaur can close horizontal distance btw theseus, it moves horizontal
        if self.grid.t_pos.1 < self.grid.m_pos.1 && 
            !self.is_wall(self.grid.m_pos.0, self.grid.m_pos.1 - 1) && 
            !self.is_goal(self.grid.m_pos.0, self.grid.m_pos.1 - 1) 
        {
            new_pos = (self.grid.m_pos.0, self.grid.m_pos.1 - 1)
        } else if self.grid.t_pos.1 > self.grid.m_pos.1 && 
            !self.is_wall(self.grid.m_pos.0, self.grid.m_pos.1 + 1) && 
            !self.is_goal(self.grid.m_pos.0, self.grid.m_pos.1 + 1) 
        {
            new_pos = (self.grid.m_pos.0, self.grid.m_pos.1 + 1)
        } 
        //Else if minotaur can close vertical distance btw theseus, it moves vertical
        else if self.grid.t_pos.0 < self.grid.m_pos.0 && 
            !self.is_wall(self.grid.m_pos.0 - 1, self.grid.m_pos.1) && 
            !self.is_goal(self.grid.m_pos.0 - 1, self.grid.m_pos.1) 
        {
            new_pos = (self.grid.m_pos.0 - 1, self.grid.m_pos.1)
        } else if self.grid.t_pos.0 < self.grid.m_pos.0 && 
            !self.is_wall(self.grid.m_pos.0 + 1, self.grid.m_pos.1) && 
            !self.is_goal(self.grid.m_pos.0 + 1, self.grid.m_pos.1) 
        {
            new_pos = (self.grid.m_pos.0 + 1, self.grid.m_pos.1)
        } else {
            return; //else stays still
        }
        if self.is_theseus(new_pos.0, new_pos.1) {
            self.status = GameStatus::Lose; //if captured theseus, end game
            return;
        }
        //else update grid and m_pos
        self.grid.vec[self.grid.m_pos.0][self.grid.m_pos.1] = ' ';
        self.grid.vec[new_pos.0][new_pos.1] = 'M';
        self.grid.m_pos = new_pos;
    }

    // moves theseus one space based on user input
    pub fn theseus_move(&mut self, command: Command) {
        let mut new_pos: (usize, usize) = self.grid.t_pos;
        //match user input with move
        match command {
            Command::Up => { 
                if new_pos.0 == 0 {
                    return; //out of min bounds, ignore
                } 
                new_pos.0 -= 1;
            }
            Command::Down => new_pos.0 += 1,
            Command::Left => { 
                if new_pos.1 == 0 {
                    return; //out of min bounds, ignore
                } 
                new_pos.1 -= 1;
            },
            Command::Right => new_pos.1 += 1,
            Command::Skip => {},
        }
        if !self.grid.in_bounds(new_pos) { //out of max bounds, ignore
        } else if self.is_minotaur(new_pos.0, new_pos.1) {
            self.grid.t_pos = new_pos; 
            self.status = GameStatus::Lose; //user loses if moves to minotaur
        } else if self.is_goal(new_pos.0, new_pos.1) {
            self.grid.t_pos = new_pos;
            self.status = GameStatus::Win; //user wins if moves to goal
        } else if self.is_empty(new_pos.0, new_pos.1) {
            //else update grid and t_pos
            self.grid.vec[self.grid.t_pos.0][self.grid.t_pos.1] = ' ';
            self.grid.vec[new_pos.0][new_pos.1] = 'T';
            self.grid.t_pos = new_pos;
        }
    }

    // output status
    pub fn status(&self) -> GameStatus {
        return self.status;
    }
}

//Other functions perform bounds checking before using following functions
impl Game {
    /// Returns true if the given position is Theseus
    pub fn is_theseus(&self, row: usize, col: usize) -> bool {
        return self.grid.vec[row][col] == 'T'
    }
    /// Returns true if the given position is Minotaur
    pub fn is_minotaur(&self, row: usize, col: usize) -> bool {
        return self.grid.vec[row][col] == 'M'
    }
    /// Returns true if the given position is a wall
    pub fn is_wall(&self, row: usize, col: usize) -> bool {
        return self.grid.vec[row][col] == 'X'
    }
    /// Returns true if the given position is the goal
    pub fn is_goal(&self, row: usize, col: usize) -> bool {
        return self.grid.vec[row][col] == 'G'
    }
    /// Returns true if the given position is empty
    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        return self.grid.vec[row][col] == ' '
    }
}



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    /// Move one tile up
    Up,
    /// Move one tile down
    Down,
    /// Move one tile left
    Left,
    /// Move one tile right
    Right,
    /// Don't move at all
    Skip,
}

//read user input to move theseus
pub fn input(stdin: impl io::Read + io::BufRead) -> Option<Command> {
    let line = stdin.lines().next().unwrap().unwrap(); //get user input
    let input_chr;
    //read first char of input 
    match line.chars().next() {
        Some(chr) => input_chr = chr.to_ascii_lowercase(),
        None => return Some(Command::Skip)
    }
    //match user input
    match input_chr {
        'w' => return Some(Command::Up),
        'a' => return Some(Command::Left),
        's' => return Some(Command::Down),
        'd' => return Some(Command::Right),
        _ => return Some(Command::Skip)
    }
}
