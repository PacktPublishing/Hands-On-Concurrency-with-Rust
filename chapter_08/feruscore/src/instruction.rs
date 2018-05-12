use rand::{thread_rng, Rng};
use std::io;
use std::io::Write;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum OpCode {
    Dat,
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Jmp,
    Jmz,
    Jmn,
    Djn,
    Spl,
    Cmp,
    Slt,
}

impl OpCode {
    pub fn random() -> OpCode {
        match thread_rng().gen_range(0, 14) {
            0 => OpCode::Dat,
            1 => OpCode::Mov,
            2 => OpCode::Add,
            3 => OpCode::Sub,
            4 => OpCode::Mul,
            5 => OpCode::Div,
            6 => OpCode::Mod,
            7 => OpCode::Jmp,
            8 => OpCode::Jmz,
            9 => OpCode::Jmn,
            10 => OpCode::Djn,
            11 => OpCode::Spl,
            12 => OpCode::Cmp,
            13 => OpCode::Slt,
            _ => unreachable!(),
        }
    }

    pub fn total() -> u32 {
        14
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        match *self {
            OpCode::Dat => w.write(b"DAT"),
            OpCode::Mov => w.write(b"MOV"),
            OpCode::Add => w.write(b"ADD"),
            OpCode::Sub => w.write(b"SUB"),
            OpCode::Mul => w.write(b"MUL"),
            OpCode::Div => w.write(b"DIV"),
            OpCode::Mod => w.write(b"MOD"),
            OpCode::Jmp => w.write(b"JMP"),
            OpCode::Jmz => w.write(b"JMZ"),
            OpCode::Jmn => w.write(b"JMN"),
            OpCode::Djn => w.write(b"DJN"),
            OpCode::Spl => w.write(b"SPL"),
            OpCode::Cmp => w.write(b"CMP"),
            OpCode::Slt => w.write(b"SLT"),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Modifier {
    A,
    B,
    AB,
    BA,
    F,
    X,
    I,
}

impl Modifier {
    pub fn random() -> Modifier {
        match thread_rng().gen_range(0, 7) {
            0 => Modifier::A,
            1 => Modifier::B,
            2 => Modifier::AB,
            3 => Modifier::BA,
            4 => Modifier::F,
            5 => Modifier::X,
            6 => Modifier::I,
            _ => unreachable!(),
        }
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        match *self {
            Modifier::A => w.write(b"A"),
            Modifier::B => w.write(b"B"),
            Modifier::AB => w.write(b"AB"),
            Modifier::BA => w.write(b"BA"),
            Modifier::F => w.write(b"F"),
            Modifier::X => w.write(b"X"),
            Modifier::I => w.write(b"I"),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Mode {
    // #
    Immediate,
    // $
    Direct,
    // * (A-field), @ (B-field)
    Indirect,
    // { (A-field), < (B-field)
    Decrement,
    // } (A-field), > (B-field)
    Increment,
}

impl Mode {
    pub fn random() -> Mode {
        match thread_rng().gen_range(0, 5) {
            0 => Mode::Immediate,
            1 => Mode::Direct,
            2 => Mode::Indirect,
            3 => Mode::Decrement,
            4 => Mode::Indirect,
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Offset {
    pub offset: i16,
}

impl Offset {
    pub fn random(core_size: u16) -> Offset {
        let mut offset: i16 = thread_rng().gen_range(0, core_size as i16);
        if thread_rng().gen::<bool>() {
            offset = -offset;
        }
        Offset { offset }
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        w.write(format!("{}", self.offset).as_bytes())
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Instruction {
    pub opcode: OpCode,
    pub modifier: Modifier,
    pub a_mode: Mode,
    pub a_offset: Offset,
    pub b_mode: Mode,
    pub b_offset: Offset,
}

impl Instruction {
    pub fn random(core_size: u16) -> Instruction {
        Instruction {
            opcode: OpCode::random(),
            modifier: Modifier::random(),
            a_mode: Mode::random(),
            a_offset: Offset::random(core_size / 32),
            b_mode: Mode::random(),
            b_offset: Offset::random(core_size / 32),
        }
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        let mut total_written = 0;
        self.opcode.serialize(w)?;
        total_written += w.write(b".")?;
        self.modifier.serialize(w)?;
        total_written += w.write(b" ")?;
        total_written += match self.a_mode {
            Mode::Immediate => w.write(b"#")?,
            Mode::Direct => w.write(b"$")?,
            Mode::Indirect => w.write(b"*")?,
            Mode::Decrement => w.write(b"{")?,
            Mode::Increment => w.write(b"}")?,
        };
        self.a_offset.serialize(w)?;
        total_written += w.write(b", ")?;
        total_written += match self.b_mode {
            Mode::Immediate => w.write(b"#")?,
            Mode::Direct => w.write(b"$")?,
            Mode::Indirect => w.write(b"@")?,
            Mode::Decrement => w.write(b"<")?,
            Mode::Increment => w.write(b">")?,
        };
        total_written += self.b_offset.serialize(w)?;
        total_written += w.write(b"\n")?;
        Ok(total_written)
    }
}
