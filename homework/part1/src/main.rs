use std::{env, fs, io::Read};

use anyhow::Error;

struct State {
    registers: [u16; 8],
}
impl State {
    fn new() -> Self {
        Self {
            registers: [0, 1, 2, 3, 4, 5, 6, 7],
        }
    }
}

struct StandardInstruction {
    v: u16,
}
impl StandardInstruction {
    const MASK_OPCODE: u16 = 0xfc00;
    const MASK_D: u16 = 0x0200;
    const MASK_W: u16 = 0x0100;
    const MASK_MODE: u16 = 0x00c0;
    const MASK_REG: u16 = 0x0038;
    const MASK_RM: u16 = 0x0007;

    fn new(v: u16) -> Self {
        Self { v }
    }

    fn read_from(mut stream: impl Read) -> Result<Self, Error> {
        let mut next_two: [u8; 2] = [0; 2];
        stream.read_exact(&mut next_two)?;
        let v = u16::from_be_bytes(next_two);

        Ok(Self { v })
    }

    fn opcode(&self) -> OpCode {
        match self.v & Self::MASK_OPCODE {
            0x8800 => OpCode::Mov,
            unknown => panic!("unknown opcode {unknown:x}"),
        }
    }

    fn reg_direction(&self) -> RegDirection {
        if (self.v & Self::MASK_D) > 0 {
            RegDirection::To
        } else {
            RegDirection::From
        }
    }

    fn op_width(&self) -> OpWidth {
        if (self.v & Self::MASK_W) > 0 {
            OpWidth::Word
        } else {
            OpWidth::Byte
        }
    }

    fn mode(&self) -> Mode {
        match (self.v & Self::MASK_MODE) >> 6 {
            0 => Mode::MemModeNoDisplacement,
            1 => Mode::MemMode8BitDisplacement,
            2 => Mode::MemMode16BitDisplacement,
            3 => Mode::Register,
            _ => panic!("unrecognized mode value"),
        }
    }

    fn reg(&self) -> Operand {
        let v = (self.v & Self::MASK_REG) >> 3;
        Operand(v.try_into().unwrap())
    }

    fn rm(&self) -> Operand {
        let v = self.v & Self::MASK_RM;
        Operand(v.try_into().unwrap())
    }
}

enum OpCode {
    Mov,
}
enum RegDirection {
    To,
    From,
}
enum OpWidth {
    Byte,
    Word,
}
enum Mode {
    MemModeNoDisplacement,
    MemMode8BitDisplacement,
    MemMode16BitDisplacement,
    Register,
}
struct Operand(u8);

fn exec_next_instruction(instr_stream: impl Read, state: &mut State) -> Result<(), Error> {
    let instr = StandardInstruction::read_from(instr_stream)?;
    println!("v: {:b} {:b}", (instr.v & 0xff00) >> 8, instr.v & 0x00ff);

    match instr.opcode() {
        OpCode::Mov => match instr.reg_direction() {
            RegDirection::To => {
                state.registers[instr.reg().0 as usize] = state.registers[instr.rm().0 as usize];
            }
            RegDirection::From => {
                state.registers[instr.rm().0 as usize] = state.registers[instr.reg().0 as usize];
            }
        },
    }

    Ok(())
}

fn register_pos_to_name(bits: u8, width: OpWidth) -> String {
    let s = match width {
        OpWidth::Byte => match bits {
            0 => "al",
            1 => "cl",
            2 => "dl",
            3 => "bl",
            4 => "ah",
            5 => "ch",
            6 => "dh",
            7 => "bh",
            _ => panic!("unknown register bits: {bits:b}"),
        },
        OpWidth::Word => match bits {
            0 => "ax",
            1 => "cx",
            2 => "dx",
            3 => "bx",
            4 => "sp",
            5 => "bp",
            6 => "si",
            7 => "di",
            _ => panic!("unknown register bits: {bits:b}"),
        },
    };
    s.to_string()
}

fn disassemble_instruction(instr: &StandardInstruction) -> Result<String, Error> {
    let opcode = match instr.opcode() {
        OpCode::Mov => "mov",
    };

    let (dst, src) = match instr.opcode() {
        OpCode::Mov => {
            let reg = register_pos_to_name(instr.reg().0, instr.op_width());
            let rm = register_pos_to_name(instr.rm().0, instr.op_width());
            match instr.reg_direction() {
                RegDirection::To => (reg, rm),
                RegDirection::From => (rm, reg),
            }
        }
    };

    let output = format!("{opcode} {dst}, {src}");
    Ok(output)
}

fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();
    let filepath = args.get(1).expect("A binary asm filepath is required");
    let mut file = fs::File::open(filepath)?;

    println!("; {filepath} disassembly");
    println!("bits 16\n");
    while let Ok(instr) = StandardInstruction::read_from(&mut file) {
        let line = disassemble_instruction(&instr)?;
        println!("{line}");
    }

    Ok(())
}
