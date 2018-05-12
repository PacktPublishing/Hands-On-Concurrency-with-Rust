use individual::*;
use instruction::*;

pub fn imp(core_size: u16) -> Individual {
    let mut child = IndividualBuilder::new();
    child = child.push(
        // MOV.I $0, $1
        InstructionBuilder::new(core_size)
            .opcode(OpCode::Mov)
            .modifier(Modifier::I)
            .a_mode(Mode::Direct)
            .a_field(0)
            .b_mode(Mode::Direct)
            .b_field(1)
            .freeze(),
    );

    child.freeze()
}

pub fn dwarf(core_size: u16) -> Individual {
    let mut child = IndividualBuilder::new();
    child = child.push(
        // ADD.AB  #4,  $3
        InstructionBuilder::new(core_size)
            .opcode(OpCode::Add)
            .modifier(Modifier::AB)
            .a_mode(Mode::Immediate)
            .a_field(4)
            .b_mode(Mode::Direct)
            .b_field(3)
            .freeze(),
    );
    child = child.push(
        // MOV.I   $2,  @2
        InstructionBuilder::new(core_size)
            .opcode(OpCode::Mov)
            .modifier(Modifier::I)
            .a_mode(Mode::Direct)
            .a_field(2)
            .b_mode(Mode::Indirect)
            .b_field(2)
            .freeze(),
    );
    child = child.push(
        // JMP.B   $-2, $0
        InstructionBuilder::new(core_size)
            .opcode(OpCode::Jmp)
            .modifier(Modifier::B)
            .a_mode(Mode::Direct)
            .a_field(-2)
            .b_mode(Mode::Direct)
            .b_field(0)
            .freeze(),
    );
    child = child.push(
        // DAT.F   #0,  #0
        InstructionBuilder::new(core_size)
            .opcode(OpCode::Dat)
            .modifier(Modifier::F)
            .a_mode(Mode::Immediate)
            .a_field(0)
            .b_mode(Mode::Immediate)
            .b_field(0)
            .freeze(),
    );
    child.freeze()
}
