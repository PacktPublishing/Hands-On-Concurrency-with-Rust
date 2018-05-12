use individual::Individual;
use instruction::*;

pub fn imp(chromosome_sz: u16) -> Individual {
    let mut chromosome = vec![
        // MOV.I $0, $1
        Some(Instruction {
            opcode: OpCode::Mov,
            modifier: Modifier::I,
            a_mode: Mode::Direct,
            a_offset: Offset { offset: 0 },
            b_mode: Mode::Direct,
            b_offset: Offset { offset: 1 },
        }),
    ];
    for _ in 0..(chromosome_sz - chromosome.len() as u16) {
        chromosome.push(None);
    }
    Individual { chromosome }
}

pub fn dwarf(chromosome_sz: u16) -> Individual {
    let mut chromosome = vec![
        // ADD.AB  #4,  $3
        Some(Instruction {
            opcode: OpCode::Add,
            modifier: Modifier::AB,
            a_mode: Mode::Immediate,
            a_offset: Offset { offset: 4 },
            b_mode: Mode::Direct,
            b_offset: Offset { offset: 3 },
        }),
        // MOV.I   $2,  @2
        Some(Instruction {
            opcode: OpCode::Mov,
            modifier: Modifier::I,
            a_mode: Mode::Direct,
            a_offset: Offset { offset: 2 },
            b_mode: Mode::Indirect,
            b_offset: Offset { offset: 2 },
        }),
        // JMP.B   $-2, $0
        Some(Instruction {
            opcode: OpCode::Jmp,
            modifier: Modifier::B,
            a_mode: Mode::Direct,
            a_offset: Offset { offset: -2 },
            b_mode: Mode::Direct,
            b_offset: Offset { offset: 0 },
        }),
        // DAT.F   #0,  #0
        Some(Instruction {
            opcode: OpCode::Dat,
            modifier: Modifier::F,
            a_mode: Mode::Immediate,
            a_offset: Offset { offset: 0 },
            b_mode: Mode::Immediate,
            b_offset: Offset { offset: 0 },
        }),
    ];
    for _ in 0..(chromosome_sz - chromosome.len() as u16) {
        chromosome.push(None);
    }
    Individual { chromosome }
}
