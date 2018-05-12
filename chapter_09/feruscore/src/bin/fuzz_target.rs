extern crate byteorder;
extern crate feruscore;

use byteorder::{BigEndian, ReadBytesExt};
use feruscore::individual::*;
use feruscore::instruction::*;
use feruscore::mars::*;
use std::io::{self, Cursor, Read};

#[derive(Debug)]
struct Config {
    pub rounds: u16,
    pub core_size: u16,
    pub cycles: u16,
    pub processes: u16,
    pub max_warrior_length: u16,
    pub left_chromosome_size: u16,
    pub right_chromosome_size: u16,
    pub left: Individual,
    pub right: Individual,
    pub left_pos: u16,
    pub right_pos: u16,
}

impl Config {
    pub fn new(rdr: &mut Cursor<Vec<u8>>) -> io::Result<Config> {
        let rounds = (rdr.read_u16::<BigEndian>()? % 1000).max(1);
        let core_size = (rdr.read_u16::<BigEndian>()? % 24_000).max(256);
        let cycles = (rdr.read_u16::<BigEndian>()? % 10_000).max(100);
        let processes = (rdr.read_u16::<BigEndian>()? % 1024).max(2);
        let max_warrior_length = (rdr.read_u16::<BigEndian>()? % 256).max(4);
        let left_chromosome_size = (rdr.read_u16::<BigEndian>()? % max_warrior_length).max(2);
        let right_chromosome_size = (rdr.read_u16::<BigEndian>()? % max_warrior_length).max(2);
        let left = Config::mk_individual(rdr, max_warrior_length, left_chromosome_size, core_size)?;
        let right =
            Config::mk_individual(rdr, max_warrior_length, right_chromosome_size, core_size)?;
        let left_pos =
            Config::adjust_pos(core_size, rdr.read_u16::<BigEndian>()?, max_warrior_length);
        let right_pos =
            Config::adjust_pos(core_size, rdr.read_u16::<BigEndian>()?, max_warrior_length);
        Ok(Config {
            rounds,
            core_size,
            cycles,
            processes,
            max_warrior_length,
            left_chromosome_size,
            right_chromosome_size,
            left,
            right,
            left_pos,
            right_pos,
        })
    }

    fn adjust_pos(core_size: u16, mut pos: u16, space: u16) -> u16 {
        pos %= core_size;
        if (pos + space) > core_size {
            let past = (pos + space) - core_size;
            pos - past
        } else {
            pos
        }
    }

    fn mk_individual(
        rdr: &mut Cursor<Vec<u8>>,
        max_chromosome_size: u16,
        chromosome_size: u16,
        core_size: u16,
    ) -> io::Result<Individual> {
        assert!(chromosome_size <= max_chromosome_size);
        let mut indv = IndividualBuilder::new();
        for _ in 0..(chromosome_size as usize) {
            let builder = InstructionBuilder::new(core_size);
            let a_field = rdr.read_i8()?;
            let b_field = rdr.read_i8()?;
            let a_mode = match rdr.read_u8()? % 5 {
                0 => Mode::Direct,
                1 => Mode::Immediate,
                2 => Mode::Indirect,
                3 => Mode::Decrement,
                _ => Mode::Increment,
            };
            let b_mode = match rdr.read_u8()? % 5 {
                0 => Mode::Direct,
                1 => Mode::Immediate,
                2 => Mode::Indirect,
                3 => Mode::Decrement,
                _ => Mode::Increment,
            };
            let modifier = match rdr.read_u8()? % 7 {
                0 => Modifier::F,
                1 => Modifier::A,
                2 => Modifier::B,
                3 => Modifier::AB,
                4 => Modifier::BA,
                5 => Modifier::X,
                _ => Modifier::I,
            };
            let opcode = match rdr.read_u8()? % 16 {
                0 => OpCode::Dat,   // 0
                1 => OpCode::Spl,   // 1
                2 => OpCode::Mov,   // 2
                3 => OpCode::Djn,   // 3
                4 => OpCode::Add,   // 4
                5 => OpCode::Jmz,   // 5
                6 => OpCode::Sub,   // 6
                7 => OpCode::Seq,   // 7
                8 => OpCode::Sne,   // 8
                9 => OpCode::Slt,   // 9
                10 => OpCode::Jmn,  // 10
                11 => OpCode::Jmp,  // 11
                12 => OpCode::Nop,  // 12
                13 => OpCode::Mul,  // 13
                14 => OpCode::Modm, // 14
                _ => OpCode::Div,   // 15
            };
            let inst = builder
                .a_field(a_field)
                .b_field(b_field)
                .a_mode(a_mode)
                .b_mode(b_mode)
                .modifier(modifier)
                .opcode(opcode)
                .freeze();
            indv = indv.push(inst);
        }
        Ok(indv.freeze())
    }
}

fn main() {
    let mut input: Vec<u8> = Vec::with_capacity(1024);
    let result = io::stdin().read_to_end(&mut input);
    if result.is_err() {
        return;
    }
    let mut rdr = Cursor::new(input);
    if let Ok(config) = Config::new(&mut rdr) {
        let mut mars = MarsBuilder::default()
            .core_size(config.core_size)
            .cycles(config.cycles)
            .processes(u32::from(config.processes))
            .max_warrior_length(config.max_warrior_length as u16)
            .freeze();
        mars.compete_inner(
            &config.left,
            config.left_pos,
            &config.right,
            config.right_pos,
        );
    }
}
