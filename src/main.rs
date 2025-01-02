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

    pub fn new_from_program(program: &[u16]) -> Self {
        let mut result = State::new();
        result.memory = program.to_vec();
        return result
    }
}

fn ascii_print(ascii_out: u8) -> () {
    print!("{}", ascii_out as u8 as char);
}

fn resolve_register(arg: u16) -> usize {
    match arg {
        32768 => 0,
        32769 => 1,
        32770 => 2,
        32771 => 3,
        32772 => 4,
        32773 => 5,
        32774 => 6,
        32775 => 7,
        _ => panic!("Invalid operand"),
    }
}

fn resolve_operand(state: &State, operand: u16) -> u16 {
    if operand <= 32767 {
        operand
    } else {
        state.registers[resolve_register(operand)]
    }
}

fn exec_program(state: &mut State, output: fn(u8) -> ()) -> bool {
    loop {
        let curr_instruction = state.memory.get(state.ip as usize);
        // println!("**** curr_instruction {} ****", curr_instruction);
        match curr_instruction {
            None => return false,
            Some(0) => return true,
            Some(9) => {
                let dest = state.memory[(state.ip + 1) as usize];
                let arg1 = resolve_operand(state, state.memory[(state.ip + 2) as usize]);
                let arg2 = resolve_operand(state, state.memory[(state.ip + 3) as usize]);

                state.registers[resolve_register(dest)] = arg1 + arg2;
                state.ip += 4;

            }
            Some(19) => {
                let ascii_out = state.memory[(state.ip + 1) as usize];
                output(ascii_out as u8);

                state.ip += 2;
            },
            Some(21) => {
                state.ip += 1;
            },
            Some(curr_instruction) => {
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
    let mut v_iter: std::vec::IntoIter<u8> = v.into_iter();
    let mut i = 0;
    while let Some(b1) = v_iter.next() {
        let b2 = v_iter.next();
        state.memory[i] = match b2 {
            None => panic!("unaligned memory"),
            Some(b2) => b1 as u16 | (b2 as u16) << 8,
        };
        i += 1;
    }

    exec_program(&mut state, ascii_print);
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{ascii_print, exec_program, State};

    #[test]
    fn run_test() {
        let mut state = State::new_from_program(&[0]);
        assert_eq!(exec_program(&mut state, ascii_print), true);

        // Can't use this to record test output as I want, rust compiler is too smart
        // let mut test_buffer : Vec<u8> = Vec::new();
        // let test_print = |ascii_out: u8| {
        //     test_buffer.push(ascii_out);
        // };

        // prints hello
        let mut state: State = State::new_from_program(&[19, 72, 19, 145, 19, 154, 19, 154, 19, 157, 0]);
        assert_eq!(exec_program(&mut state, ascii_print), true);

        let mut state: State = State::new_from_program(&[9, 32768, 32769, 4, 19, 32768, 0]);
        state.registers[1] = 60;
        assert_eq!(exec_program(&mut state, ascii_print), true);
        assert_eq!(state.registers[0], 64);
    }
}
