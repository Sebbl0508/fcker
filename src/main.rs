use std::io::{stdin, stdout, Read, Write};

pub static VALID_SOURCE_CHAR: &[char] = &['>', '<', '+', '-', '.', ',', '[', ']'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    /// '>'
    IncDataPtr,
    /// '<'
    DecDataPtr,
    /// '+'
    IncValue,
    /// '-'
    DecValue,
    /// '.'
    Output,
    /// ','
    GetInput,
    /// '['
    JumpForward,
    /// ']'
    JumpBackward,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
struct CPU {
    // instruction pointer
    ip: usize,
    dp: usize,
    memory: Box<[u8]>,
    code: Box<[Token]>,
    prevent_ip_inc: bool,
}

fn main() {
    let file = std::env::args()
        .nth(1)
        .expect("couldn't get first argument");

    let file_text =
        std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("couldn't read file '{}'", &file));

    let source_tokens = tokenize_source(file_text);

    let cpu = CPU::new(source_tokens);
    cpu.run();
}

fn tokenize_source<S>(input: S) -> Vec<Token>
where
    S: Into<String>,
{
    let input: String = input.into();

    input
        .chars()
        .filter_map(|c| Token::try_from(c).ok())
        .collect()
}

impl CPU {
    const DEFAULT_MEM_SIZE: usize = 30_000;

    pub fn new<C>(code: C) -> Self
    where
        C: Into<Box<[Token]>>,
    {
        Self {
            ip: 0,
            dp: 0,
            memory: vec![0; Self::DEFAULT_MEM_SIZE].into_boxed_slice(),
            code: code.into(),
            prevent_ip_inc: false,
        }
    }

    pub fn run(mut self) {
        loop {
            let instruction = self.code[self.ip];
            match instruction {
                Token::IncDataPtr => self.execute_inc_data_ptr(),
                Token::DecDataPtr => self.execute_dec_data_ptr(),
                Token::IncValue => self.execute_inc_value(),
                Token::DecValue => self.execute_dec_value(),
                Token::Output => self.execute_output(),
                Token::GetInput => self.execute_get_input(),
                Token::JumpForward => self.execute_jump_forward(),
                Token::JumpBackward => self.execute_jump_backward(),
            }

            // Some instruction doesn't want the instruction pointer
            // getting increased. So instead we'll reset the flag for the next instruction
            if self.prevent_ip_inc {
                self.prevent_ip_inc = false;
            } else {
                self.ip += 1;
            }

            if self.ip >= self.code.len() {
                return;
            }
        }
    }

    fn execute_inc_data_ptr(&mut self) {
        self.dp += 1;
        if self.dp >= self.memory.len() {
            panic!("Exceeded memory limit of {} kB", self.memory.len() / 1000);
        }
    }

    fn execute_dec_data_ptr(&mut self) {
        self.dp -= 1;
        if self.dp >= self.memory.len() {
            panic!("Tried moving the data pointer into the negative!");
        }
    }

    fn execute_inc_value(&mut self) {
        self.memory[self.dp] = self.memory[self.dp].wrapping_add(1);
    }

    fn execute_dec_value(&mut self) {
        self.memory[self.dp] = self.memory[self.dp].wrapping_sub(1);
    }

    fn execute_output(&mut self) {
        let byte = self.memory[self.dp];
        let mut stdout_lock = stdout().lock();
        stdout_lock.write_all(&[byte]).unwrap();
        stdout_lock.flush().unwrap();
    }

    fn execute_get_input(&mut self) {
        let mut buf = [0u8];
        let num_read = stdin().lock().read(&mut buf).unwrap();

        if num_read == 1 {
            self.memory[self.dp] = buf[0];
        }
    }

    fn execute_jump_forward(&mut self) {
        if self.memory[self.dp] != 0 {
            return;
        }

        self.ip = self.find_matching_arm_idx();
        self.prevent_ip_inc = true;
    }

    fn execute_jump_backward(&mut self) {
        if self.memory[self.dp] == 0 {
            return;
        }

        self.ip = self.find_matching_arm_idx();
        self.prevent_ip_inc = true;
    }

    fn find_matching_arm_idx(&mut self) -> usize {
        let forward = match self.code[self.ip] {
            Token::JumpForward => true,
            Token::JumpBackward => false,
            t => panic!("can't find matching arm for instruction '{t:?}'"),
        };

        let mut unmatched_brackets = 1;
        let mut tmp_ip = self.ip;
        loop {
            if forward {
                tmp_ip += 1;
            } else {
                tmp_ip -= 1;
            }

            if tmp_ip >= self.code.len() {
                panic!("UNMATCHED BRACKET!");
            }

            let inst = self.code[tmp_ip];

            match inst {
                Token::JumpForward if forward => unmatched_brackets += 1,
                Token::JumpBackward if !forward => unmatched_brackets += 1,
                Token::JumpForward if !forward => unmatched_brackets -= 1,
                Token::JumpBackward if forward => unmatched_brackets -= 1,
                _ => {}
            }

            if unmatched_brackets == 0 {
                return tmp_ip;
            }
        }
    }
}

impl TryFrom<char> for Token {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '>' => Ok(Token::IncDataPtr),
            '<' => Ok(Token::DecDataPtr),
            '+' => Ok(Token::IncValue),
            '-' => Ok(Token::DecValue),
            '.' => Ok(Token::Output),
            ',' => Ok(Token::GetInput),
            '[' => Ok(Token::JumpForward),
            ']' => Ok(Token::JumpBackward),
            c => Err(c),
        }
    }
}
