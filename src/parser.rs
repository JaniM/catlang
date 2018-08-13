use spec::CatCommand;
use std::iter::Peekable;

enum ReadResult {
    Ok,
    NoMatch(char),
    Done,
}

#[derive(Debug)]
pub struct Parser {
    pub commands: Vec<CatCommand>,
    whitespace_needed: bool,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            commands: vec![],
            whitespace_needed: false,
        }
    }

    pub fn parse(&mut self, text: &str) -> Result<(), String> {
        let mut chars = text.chars().peekable();
        loop {
            match self.read_one(&mut chars, &[]) {
                ReadResult::Ok => {}
                ReadResult::NoMatch(_) => {
                    return Err(format!("Unexpected character: {}", chars.peek().unwrap()))
                }
                ReadResult::Done => return Ok(()),
            }
        }
    }

    fn read_one<I: Iterator<Item = char>>(
        &mut self,
        chars: &mut Peekable<I>,
        excluded: &[char],
    ) -> ReadResult {
        let c = if let Some(c) = chars.peek() {
            *c
        } else {
            return ReadResult::Done;
        };
        if excluded.contains(&c) {
            return ReadResult::NoMatch(c);
        }
        if c.is_whitespace() {
            chars.next();
            if !self.whitespace_needed {
                self.commands.push(CatCommand::CreateString(c.to_string()));
            }
            self.whitespace_needed = false;
            return ReadResult::Ok;
        }
        self.whitespace_needed = false;

        if c == '"' {
            chars.next();
            self.read_string(chars);
        } else if c == '\'' {
            chars.next();
            self.read_char(chars);
        } else if c.is_digit(10) {
            self.read_digit(chars)
        } else if self.read_command(chars) {
        } else {
            return ReadResult::NoMatch(c);
        }
        ReadResult::Ok
    }

    fn read_digit<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) {
        let mut digits: Vec<i64> = vec![];
        loop {
            let c = if let Some(c) = chars.peek() {
                *c
            } else {
                break;
            };
            if let Some(d) = c.to_digit(10) {
                digits.push(d as i64);
                chars.next();
            } else {
                self.whitespace_needed = true;
                break;
            }
        }
        let num = digits
            .iter()
            .enumerate()
            .map(|(i, v)| 10i64.pow((digits.len() - i - 1) as u32) * v)
            .sum();
        self.commands.push(CatCommand::CreateInteger(num))
    }

    fn read_string<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) {
        let mut buffer: Vec<char> = vec![];
        loop {
            let mut c = if let Some(c) = chars.next() {
                c
            } else {
                break;
            };
            if c == '\\' {
                c = if let Some(c) = chars.next() {
                    c
                } else {
                    break;
                };
            } else if c == '"' {
                break;
            }
            buffer.push(c);
        }
        self.commands
            .push(CatCommand::CreateString(buffer.into_iter().collect()));
    }

    fn read_char<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) {
        let c = if let Some(c) = chars.next() {
            c
        } else {
            return;
        };
        self.commands.push(CatCommand::CreateString(c.to_string()));
    }

    fn read_command<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) -> bool {
        let c = if let Some(c) = chars.peek() {
            *c
        } else {
            return false;
        };
        let mut no_next = false;
        let cmd = match c {
            '[' => CatCommand::StartBlock,
            ']' => CatCommand::CloseBlock,
            '(' => CatCommand::StartBlock,
            ')' => CatCommand::CloseBlock,
            '`' => {
                no_next = true;
                chars.next();
                self.read_command(chars);
                let f = self.commands.pop().unwrap();
                CatCommand::CreateCommand(Box::new(f))
            }
            '+' => CatCommand::Add,
            '*' => CatCommand::Multiply,
            'R' => CatCommand::ReadLine,
            'W' => CatCommand::WriteLine,
            'w' => CatCommand::Write,
            'M' => {
                if !self.read_command_block(chars) {
                    return false;
                }
                no_next = true;
                CatCommand::Map
            }
            'F' => {
                if !self.read_command_block(chars) {
                    return false;
                }
                no_next = true;
                CatCommand::ForEach
            }
            '#' => {
                if !self.read_command_block(chars) {
                    return false;
                }
                no_next = true;
                CatCommand::Repeat
            }
            '!' => CatCommand::Execute,
            'S' => CatCommand::Split,
            'I' => CatCommand::ToInteger,
            'r' => CatCommand::Range,
            ':' => CatCommand::Duplicate,
            ';' => CatCommand::DuplicateSecond,
            '_' => CatCommand::Drop,
            'x' => CatCommand::Rotate(2),
            'X' => CatCommand::Rotate(3),
            'p' => CatCommand::PushSide,
            'P' => CatCommand::PopSide,
            '~' => CatCommand::ConsumeSide,
            'J' => CatCommand::Join,
            _ => return false,
        };
        if !no_next {
            chars.next();
        }
        self.commands.push(cmd);
        if c == ')' {
            self.commands.push(CatCommand::ExecuteScoped);
        }
        return true;
    }

    fn read_command_block<I: Iterator<Item = char>>(&mut self, chars: &mut Peekable<I>) -> bool {
        chars.next();
        if let Some(&c) = chars.peek() {
            match c {
                '$' => {
                    chars.next();
                    return true;
                }
                ')' => return true,
                ']' => return true,
                _ => {}
            }
        }
        self.commands.push(CatCommand::StartBlock);
        loop {
            match self.read_one(chars, &[')', ']']) {
                ReadResult::Ok => {}
                ReadResult::NoMatch('$') => {
                    chars.next();
                    break;
                }
                ReadResult::NoMatch(')') => break,
                ReadResult::NoMatch(']') => break,
                ReadResult::NoMatch(_) => return false,
                ReadResult::Done => break,
            }
        }
        self.commands.push(CatCommand::CloseBlock);
        true
    }
}
