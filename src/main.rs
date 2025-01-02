use std::{fs, iter};

// OK so we need a VM state
// actually numbers are 15-bit but whatever.
struct State {
    memory: Vec<u16>,
    registers: [u16; 8], // static size
    stack: Vec<u16>,
    ip: u16, // instruction pointer
}

impl State {
    pub fn new() -> Self {
        State{
            memory: Vec::new(),
            registers: [0, 0, 0, 0, 0, 0, 0, 0],
            stack: Vec::new(),
            ip: 0,
        }
    }
}

fn exec_program(state: &mut State) {
    loop {
        let mut curr_instruction = state.memory[state.ip as usize];
        // println!("**** curr_instruction {} ****", curr_instruction);
        match curr_instruction {
            0 => return,
            19 => {
                let ascii_out = state.memory[(state.ip + 1) as usize];
                // println!("**** PRINT ****");
                print!("{}", ascii_out as u8 as char);

                state.ip += 2;
            },
            21 => {
                // println!("**** NOOP ****");
                state.ip += 1;
            },
            _ => {
                todo!("unimplemented opcode {}", curr_instruction);
            },
        }
    }
}

fn main() -> std::io::Result<()> {
    // println!("Hello, world!");
    let mut state = State::new();
    let v = fs::read("challenge.bin")?;
    state.memory = vec![0; v.len() / 2];
    let mut v_iter = v.into_iter();
    let mut i = 0;
    while let Some(b1) = v_iter.next() {
        let b2 = v_iter.next();
        state.memory[i] = match b2 {
            None => panic!("unaligned memory"),
            Some(b2) => b1 as u16 | (b2 as u16) << 8,
        };
        i += 1;
    }

    exec_program(&mut state);
    Ok(())
}
