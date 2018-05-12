use rand::{Rand, Rng};
use std::io::{self, Write};

const AMODE_MASK: u16 = 0b0000_0000_0000_0111;
const BMODE_MASK: u16 = 0b0000_0000_0011_1000;
const MODIFIER_MASK: u16 = 0b0000_0001_1100_0000;
const OP_CODE_MASK: u16 = 0b0011_1110_0000_0000;
const FLAG_MASK: u16 = 0b1100_0000_0000_0000;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Flag {
    Normal, // 0
    Start,  // 1
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Rand)]
pub enum OpCode {
    Dat,  // 0
    Spl,  // 1
    Mov,  // 2
    Djn,  // 3
    Add,  // 4
    Jmz,  // 5
    Sub,  // 6
    Seq,  // 7
    Sne,  // 8
    Slt,  // 9
    Jmn,  // 10
    Jmp,  // 11
    Nop,  // 12
    Mul,  // 13
    Modm, // 14
    Div,  // 15
}

impl OpCode {
    pub fn total() -> u32 {
        16
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Rand)]
pub enum Modifier {
    F,
    A,
    B,
    AB,
    BA,
    X,
    I,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Rand)]
pub enum Mode {
    // $
    Direct,
    // #
    Immediate,
    // * (A-field), @ (B-field)
    Indirect,
    // { (A-field), < (B-field)
    Decrement,
    // } (A-field), > (B-field)
    Increment,
}

pub enum ExhaustMode {
    Direct,    // Mode::Direct
    Immediate, // Mode::Immediate
    BIndirect, // Mode::Indirect for b-field
    BPredec,   // Mode::Decrement for b-field
    BPostinc,  // Mode::Increment for b-field
    AIndirect, // Mode::Indirect for a-field
    APredec,   // Mode::Decrement for a-field
    APostinc,  // Mode::Increment for b-field
}

pub struct InstructionBuilder {
    core_size: u16,
    ins: u16,
    a: u16,
    b: u16,
}

impl InstructionBuilder {
    pub fn new(core_size: u16) -> Self {
        InstructionBuilder {
            core_size,
            ins: 0_u16,
            a: 0,
            b: 0,
        }
    }

    pub fn modifier(mut self, modifier: Modifier) -> Self {
        let modifier_no = modifier as u16;
        self.ins &= !MODIFIER_MASK;
        self.ins |= modifier_no << MODIFIER_MASK.trailing_zeros();
        self
    }

    pub fn opcode(mut self, opcode: OpCode) -> Self {
        let opcode_no = opcode as u16;
        self.ins &= !OP_CODE_MASK;
        self.ins |= opcode_no << OP_CODE_MASK.trailing_zeros();
        self
    }

    pub fn a_mode(mut self, mode: Mode) -> Self {
        let exhaust_mode = match mode {
            Mode::Direct => ExhaustMode::Direct,
            Mode::Immediate => ExhaustMode::Immediate,
            Mode::Indirect => ExhaustMode::AIndirect,
            Mode::Decrement => ExhaustMode::APredec,
            Mode::Increment => ExhaustMode::APostinc,
        };
        let mode_no = exhaust_mode as u16;
        self.ins &= !AMODE_MASK;
        self.ins |= mode_no << AMODE_MASK.trailing_zeros();
        self
    }

    pub fn b_mode(mut self, mode: Mode) -> Self {
        let exhaust_mode = match mode {
            Mode::Direct => ExhaustMode::Direct,
            Mode::Immediate => ExhaustMode::Immediate,
            Mode::Indirect => ExhaustMode::BIndirect,
            Mode::Decrement => ExhaustMode::BPredec,
            Mode::Increment => ExhaustMode::BPostinc,
        };
        let mode_no = exhaust_mode as u16;
        self.ins &= !BMODE_MASK;
        self.ins |= mode_no << BMODE_MASK.trailing_zeros();
        self
    }

    pub fn freeze(self) -> Instruction {
        Instruction {
            ins: self.ins,
            a: self.a,
            b: self.b,
        }
    }

    pub fn a_field(mut self, offset: i8) -> Self {
        if offset.is_negative() {
            self.a = (i16::from(offset) + (self.core_size as i16)) as u16;
        } else {
            self.a = offset as u16;
        }
        self
    }

    pub fn b_field(mut self, offset: i8) -> Self {
        if offset.is_negative() {
            self.b = (i16::from(offset) + (self.core_size as i16)) as u16;
        } else {
            self.b = offset as u16;
        }
        self
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Instruction {
    // The layout of the `ins` field is as follows:
    //
    // bit         15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
    // field   | flags | |- op-code  -| |-.mod-| |b-mode| |a-mode|
    //
    // feruscore ignores flags. The only one exhaust supports is a START
    // flag. Our start is always the first instruction.
    a: u16,
    b: u16,
    ins: u16,
}

impl Instruction {
    pub fn thaw(self, core_size: u16) -> InstructionBuilder {
        InstructionBuilder {
            core_size,
            ins: self.ins,
            a: self.a,
            b: self.b,
        }
    }

    pub fn start(&mut self) -> () {
        let flag_no = Flag::Start as u16;
        self.ins &= !FLAG_MASK;
        self.ins |= flag_no << FLAG_MASK.trailing_zeros();
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        let mut total_written = 0;
        total_written += match self.opcode() {
            OpCode::Dat => w.write(b"DAT")?,
            OpCode::Spl => w.write(b"SPL")?,
            OpCode::Mov => w.write(b"MOV")?,
            OpCode::Djn => w.write(b"DJN")?,
            OpCode::Add => w.write(b"ADD")?,
            OpCode::Jmz => w.write(b"JMZ")?,
            OpCode::Sub => w.write(b"SUB")?,
            OpCode::Seq => w.write(b"SEQ")?,
            OpCode::Sne => w.write(b"SNE")?,
            OpCode::Slt => w.write(b"SLT")?,
            OpCode::Jmn => w.write(b"JMN")?,
            OpCode::Jmp => w.write(b"JMP")?,
            OpCode::Nop => w.write(b"NOP")?,
            OpCode::Mul => w.write(b"MUL")?,
            OpCode::Modm => w.write(b"MODM")?,
            OpCode::Div => w.write(b"DIV")?,
        };
        total_written += w.write(b".")?;
        total_written += match self.modifier() {
            Modifier::F => w.write(b"F")?,
            Modifier::A => w.write(b"A")?,
            Modifier::B => w.write(b"B")?,
            Modifier::AB => w.write(b"AB")?,
            Modifier::BA => w.write(b"BA")?,
            Modifier::X => w.write(b"X")?,
            Modifier::I => w.write(b"I")?,
        };
        total_written += w.write(b" ")?;
        total_written += match self.a_mode() {
            Mode::Immediate => w.write(b"#")?,
            Mode::Direct => w.write(b"$")?,
            Mode::Indirect => w.write(b"*")?,
            Mode::Decrement => w.write(b"{")?,
            Mode::Increment => w.write(b"}")?,
        };
        total_written += w.write(format!("{}", self.a).as_bytes())?;
        total_written += w.write(b", ")?;
        total_written += match self.b_mode() {
            Mode::Immediate => w.write(b"#")?,
            Mode::Direct => w.write(b"$")?,
            Mode::Indirect => w.write(b"@")?,
            Mode::Decrement => w.write(b"<")?,
            Mode::Increment => w.write(b">")?,
        };
        total_written += w.write(format!("{}", self.b).as_bytes())?;
        total_written += w.write(b"\n")?;
        Ok(total_written)
    }

    pub fn a_mode(&self) -> Mode {
        let shift: u32 = AMODE_MASK.trailing_zeros();
        let mode_no = (self.ins & AMODE_MASK) >> shift;
        match mode_no {
            0 => Mode::Direct,
            1 => Mode::Immediate,
            2 | 3 | 4 => unreachable!(),
            5 => Mode::Indirect,
            6 => Mode::Decrement,
            7 => Mode::Increment,
            _ => unreachable!(),
        }
    }

    pub fn b_mode(&self) -> Mode {
        let shift: u32 = BMODE_MASK.trailing_zeros();
        let mode_no = (self.ins & BMODE_MASK) >> shift;
        match mode_no {
            0 => Mode::Direct,
            1 => Mode::Immediate,
            2 => Mode::Indirect,
            3 => Mode::Decrement,
            4 => Mode::Increment,
            _ => unreachable!(),
        }
    }

    pub fn a(&self) -> u16 {
        self.a
    }

    pub fn b(&self) -> u16 {
        self.b
    }

    pub fn modifier(&self) -> Modifier {
        let shift: u32 = MODIFIER_MASK.trailing_zeros();
        let modifier_no = (self.ins & MODIFIER_MASK) >> shift;
        match modifier_no {
            0 => Modifier::F,
            1 => Modifier::A,
            2 => Modifier::B,
            3 => Modifier::AB,
            4 => Modifier::BA,
            5 => Modifier::X,
            6 => Modifier::I,
            _ => unreachable!(),
        }
    }

    pub fn opcode(&self) -> OpCode {
        let shift: u32 = OP_CODE_MASK.trailing_zeros();
        let opcode_no = (self.ins & OP_CODE_MASK) >> shift;
        match opcode_no {
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
            15 => OpCode::Div,  // 15
            _ => panic!(format!("NOT AN OPCODE: {}", opcode_no)),
        }
    }
}

impl Rand for Instruction {
    fn rand<R: Rng>(rng: &mut R) -> Instruction {
        // TODO figure out a way of making sure 8000 is in fact the core size
        InstructionBuilder::new(8000)
            .opcode(rng.gen::<OpCode>())
            .modifier(rng.gen::<Modifier>())
            .a_mode(rng.gen::<Mode>())
            .a_field(rng.gen::<i8>())
            .b_mode(rng.gen::<Mode>())
            .b_field(rng.gen::<i8>())
            .freeze()
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

    impl Arbitrary for Modifier {
        fn arbitrary<G>(g: &mut G) -> Modifier
        where
            G: Gen,
        {
            match g.gen_range(0, 7) {
                0 => Modifier::F,
                1 => Modifier::A,
                2 => Modifier::B,
                3 => Modifier::AB,
                4 => Modifier::BA,
                5 => Modifier::X,
                6 => Modifier::I,
                _ => unreachable!(),
            }
        }
    }

    impl Arbitrary for OpCode {
        fn arbitrary<G>(g: &mut G) -> OpCode
        where
            G: Gen,
        {
            match g.gen_range(0, 16) {
                0 => OpCode::Dat,
                1 => OpCode::Spl,
                2 => OpCode::Mov,
                3 => OpCode::Djn,
                4 => OpCode::Add,
                5 => OpCode::Jmz,
                6 => OpCode::Sub,
                7 => OpCode::Seq,
                8 => OpCode::Sne,
                9 => OpCode::Slt,
                10 => OpCode::Jmn,
                11 => OpCode::Jmp,
                12 => OpCode::Nop,
                13 => OpCode::Mul,
                14 => OpCode::Modm,
                15 => OpCode::Div,
                _ => unreachable!(),
            }
        }
    }

    impl Arbitrary for Mode {
        fn arbitrary<G>(g: &mut G) -> Mode
        where
            G: Gen,
        {
            match g.gen_range(0, 5) {
                0 => Mode::Direct,
                1 => Mode::Immediate,
                2 => Mode::Indirect,
                3 => Mode::Decrement,
                4 => Mode::Indirect,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone)]
    enum ApplyOrder {
        AMode,
        BMode,
        A,
        B,
        Modifier,
        OpCode,
    }

    impl Arbitrary for ApplyOrder {
        fn arbitrary<G>(g: &mut G) -> ApplyOrder
        where
            G: Gen,
        {
            match g.gen_range(0, 6) {
                0 => ApplyOrder::AMode,
                1 => ApplyOrder::BMode,
                2 => ApplyOrder::A,
                3 => ApplyOrder::B,
                4 => ApplyOrder::Modifier,
                5 => ApplyOrder::OpCode,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Application {
        orders: Vec<ApplyOrder>,
        a: Vec<i8>,
        b: Vec<i8>,
        a_mode: Vec<Mode>,
        b_mode: Vec<Mode>,
        modifier: Vec<Modifier>,
        opcode: Vec<OpCode>,
    }

    impl Arbitrary for Application {
        fn arbitrary<G>(g: &mut G) -> Application
        where
            G: Gen,
        {
            let max = 100;
            Application {
                orders: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| ApplyOrder::arbitrary(g))
                    .collect(),
                a: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| i8::arbitrary(g))
                    .collect(),
                b: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| i8::arbitrary(g))
                    .collect(),
                a_mode: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| Mode::arbitrary(g))
                    .collect(),
                b_mode: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| Mode::arbitrary(g))
                    .collect(),
                modifier: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| Modifier::arbitrary(g))
                    .collect(),
                opcode: (0..g.gen_range(0, max))
                    .into_iter()
                    .map(|_| OpCode::arbitrary(g))
                    .collect(),
            }
        }
    }

    fn normalize_field(field: i8, core_size: u16) -> u16 {
        if field.is_negative() {
            (i16::from(field) + (core_size as i16)) as u16
        } else {
            field as u16
        }
    }

    #[test]
    fn instruction_identity() {
        fn inner(
            core_size: u16,
            mut application: Application,
            orders: Vec<ApplyOrder>,
        ) -> TestResult {
            if core_size == 0 {
                return TestResult::discard();
            }
            let mut instruction = InstructionBuilder::new(core_size).freeze();

            for order in orders.into_iter() {
                match order {
                    ApplyOrder::AMode => {
                        if let Some(mode) = application.a_mode.pop() {
                            let builder = instruction.thaw(core_size);
                            instruction = builder.a_mode(mode).freeze();
                            assert_eq!(instruction.a_mode(), mode);
                        }
                    }
                    ApplyOrder::BMode => {
                        if let Some(mode) = application.b_mode.pop() {
                            let builder = instruction.thaw(core_size);
                            instruction = builder.b_mode(mode).freeze();
                            assert_eq!(instruction.b_mode(), mode);
                        }
                    }
                    ApplyOrder::A => {
                        if let Some(a) = application.a.pop() {
                            let builder = instruction.thaw(core_size);
                            instruction = builder.a_field(a).freeze();
                            assert_eq!(instruction.a(), normalize_field(a, core_size));
                        }
                    }
                    ApplyOrder::B => {
                        if let Some(b) = application.b.pop() {
                            let builder = instruction.thaw(core_size);
                            instruction = builder.b_field(b).freeze();
                            assert_eq!(instruction.b(), normalize_field(b, core_size));
                        }
                    }
                    ApplyOrder::Modifier => {
                        if let Some(modifier) = application.modifier.pop() {
                            let builder = instruction.thaw(core_size);
                            instruction = builder.modifier(modifier).freeze();
                            assert_eq!(instruction.modifier(), modifier);
                        }
                    }
                    ApplyOrder::OpCode => {
                        if let Some(opcode) = application.opcode.pop() {
                            let builder = instruction.thaw(core_size);
                            instruction = builder.opcode(opcode).freeze();
                            assert_eq!(instruction.opcode(), opcode);
                        }
                    }
                }
            }
            TestResult::passed()
        }
        QuickCheck::new().quickcheck(inner as fn(u16, Application, Vec<ApplyOrder>) -> TestResult);
    }
}
