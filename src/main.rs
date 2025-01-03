use std::{fs, iter};
use std::io::{stdout, stdin, Write};

// OK so we need a VM state
// actually numbers are 15-bit but whatever.
struct State {
    memory: Vec<u16>,
    registers: [u16; 8], // static size
    stack: Vec<u16>,
    ip: usize, // instruction pointer
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
    // println!("****");
    // println!("{}", ascii_out);
    print!("{}", ascii_out as u8 as char);
    // println!();
    // println!("****");
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
        _ => panic!("Invalid operand {}", arg),
    }
}

fn resolve_operand(state: &State, operand: u16) -> u16 {
    if operand <= 32767 {
        operand
    } else {
        state.registers[resolve_register(operand)]
    }
}

fn arity(opcode: u16) -> usize {
    match opcode {
        0 => 0,
        1 => 2,
        2 => 1,
        3 => 1,
        4 => 3,
        5 => 3,
        6 => 1, // jump
        7 => 2, // jump
        8 => 2, // jump
        9 => 3,
        10 => 3,
        11 => 3,
        12 => 3,
        13 => 3,
        14 => 2,
        15 => 2,
        16 => 2,
        17 => 1,
        18 => 0,
        19 => 1,
        20 => 1,
        21 => 0,
        _ => panic!("Invalid opcode"),
    }
}

fn exec_program(state: &mut State, output: fn(u8) -> ()) -> bool {
    let mut input_buff = String::new();
    let mut input_remaining: Result<usize, _>;

    loop {
        let curr_instruction = state.memory.get(state.ip);
        let opcode_arity: Option<usize> = curr_instruction.map(|x| arity(*x));
        let mut jump_instruction : Option<usize> = None;

        match curr_instruction {
            None => return false,
            Some(0) => return true,
            Some(1) => {
                let dest = state.memory[state.ip + 1];
                let arg = resolve_operand(state, state.memory[state.ip + 2]);
                state.registers[resolve_register(dest)] = arg;
            }
            Some(2) => {
                state.stack.push(resolve_operand(state, state.memory[state.ip + 1]));
            }
            Some(3) => {
                let dest = resolve_register(state.memory[state.ip + 1]);
                match state.stack.pop() {
                    None => {
                        panic!("Empty stack");
                    },
                    Some(val) => {
                        state.registers[dest] = val;
                    }
                }
            }
            Some(4) | Some(5) => {
                let dest = resolve_register(state.memory[state.ip + 1]);
                let arg1 = resolve_operand(state, state.memory[state.ip + 2]);
                let arg2 = resolve_operand(state, state.memory[state.ip + 3]);
                state.registers[dest] = if curr_instruction == Some(&4) {
                    if arg1 == arg2 { 1 } else { 0 }
                } else {
                    if arg1 > arg2 { 1 } else { 0 }
                }
            }
            Some(6) => {
                let dest = resolve_operand(state, state.memory[state.ip + 1]);
                jump_instruction = Some(dest as usize);
            },
            Some(7) | Some(8) => {
                let test = resolve_operand(state, state.memory[state.ip + 1]);
                let dest = resolve_operand(state, state.memory[state.ip + 2]);
                if (curr_instruction == Some(&7) && test != 0) ||
                   (curr_instruction == Some(&8) && test == 0) {
                    jump_instruction = Some(dest as usize);
                }
            }
            Some(9) | Some(10) | Some(11) | Some(12) | Some(13) => {
                let dest = state.memory[state.ip + 1];
                let arg1 = resolve_operand(state, state.memory[state.ip + 2]);
                let arg2 = resolve_operand(state, state.memory[state.ip + 3]);

                state.registers[resolve_register(dest)] = match curr_instruction {
                    Some(9) => (arg1 + arg2) & 0x7FFF,
                    Some(10) => arg1.overflowing_mul(arg2).0 & 0x7FFF,
                    Some(11) => arg1 % arg2,
                    Some(12) => arg1 & arg2,
                    Some(13) => arg1 | arg2,
                    _ => todo!()
                }
            }
            Some(14) => {
                let dest = state.memory[state.ip + 1];
                let arg = resolve_operand(state, state.memory[state.ip + 2]);

                state.registers[resolve_register(dest)] = !arg & 0x7FFF;
            }
            Some(15) => {
                // rmem 15 a b
                // read memory at address <b> and write it to <a>
                let dest = resolve_register(state.memory[state.ip + 1]);
                let arg = resolve_operand(state, state.memory[state.ip + 2]);

                state.registers[dest] = state.memory[arg as usize];
            }
            Some(16) => {
                // wmem 15 a b
                // write the value from <b> into memory at address <a>
                let dest = resolve_operand(state, state.memory[state.ip + 1]) as usize;
                let arg = resolve_operand(state, state.memory[state.ip + 2]);

                state.memory[dest] = arg;
            }
            Some(17) => {
                // call: 17 a
                // write the address of the next instruction to the stack and jump to <a>
                let next_instruction = state.ip + 2;
                let jump_dest = resolve_operand(state, state.memory[state.ip + 1]);
                state.stack.push(next_instruction as u16);
                jump_instruction = Some(jump_dest as usize);
            }
            Some(18) => {
                // ret: 18
                //   remove the top element from the stack and jump to it; empty stack = halt
                match state.stack.pop() {
                    None => panic!("Popped empty stack"),
                    Some(instr) => jump_instruction = Some(instr as usize)
                }
            }
            Some(19) => {
                let ascii_out = resolve_operand(state, state.memory[state.ip + 1]);
                output(ascii_out as u8);
            }
            Some(20) => {
// in: 20 a
// read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard instead of having to figure out how to read individual characters
                if input_buff.is_empty() {
                    input_remaining = stdin().read_line(&mut input_buff);
                }
                let first_char: Option<char>;
                input_buff = {
                    let mut chars = input_buff.chars();
                    first_char = chars.next();
                    chars.as_str().to_string()
                };
                let dest = resolve_register(state.memory[state.ip + 1]) as usize;
                state.registers[dest] = first_char.unwrap() as u16;
            }
            Some(21) => {
                // arity handles for us
            }
            Some(curr_instruction) => {
                todo!("unimplemented opcode {}", curr_instruction);
            }
        }
        // is this the best we can do?
        match jump_instruction {
            None => state.ip += opcode_arity.unwrap() + 1,
            Some(next) => state.ip = next
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
