pub enum Instruction {
	// 8 bit arithmetic
	ADD(ArithmeticTarget),
	ADC(ArithmeticTarget),
	SUB(ArithmeticTarget),
	SBC(ArithmeticTarget),
	AND(ArithmeticTarget),
	OR(ArithmeticTarget),
	XOR(ArithmeticTarget),
	CP(ArithmeticTarget),
	INC(IncDecTarget),
	DEC(IncDecTarget),
	// 16 bit arithmetic
	ADDHL(HLTarget),
	// jump instructions
	JP(JumpTest),
	// load instructions
	LD(LoadType),
	// stack instructions
	PUSH(StackTarget),
	POP(StackTarget),
}

impl Instruction {
	pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
		if prefixed {
			Instruction::from_byte_prefixed(byte)
		} else {
			Instruction::from_byte_not_prefixed(byte)
		}
	}

	fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
		match byte {
			_ => None,
		}
	}

	fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
		match byte {
			//8-bit Arithmetic
			// ADD
			0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
			0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
			0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
			0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
			0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
			0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
			0x86 => Some(Instruction::ADD(ArithmeticTarget::HLI)),
			0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),
			// ADC (Add with Carry)
			0x88 => Some(Instruction::ADC(ArithmeticTarget::B)),
			0x89 => Some(Instruction::ADC(ArithmeticTarget::C)),
			0x8A => Some(Instruction::ADC(ArithmeticTarget::D)),
			0x8B => Some(Instruction::ADC(ArithmeticTarget::E)),
			0x8C => Some(Instruction::ADC(ArithmeticTarget::H)),
			0x8D => Some(Instruction::ADC(ArithmeticTarget::L)),
			0x8E => Some(Instruction::ADC(ArithmeticTarget::HLI)),
			0x8F => Some(Instruction::ADC(ArithmeticTarget::A)),
			// SUB
			0x90 => Some(Instruction::SUB(ArithmeticTarget::B)),
			0x91 => Some(Instruction::SUB(ArithmeticTarget::C)),
			0x92 => Some(Instruction::SUB(ArithmeticTarget::D)),
			0x93 => Some(Instruction::SUB(ArithmeticTarget::E)),
			0x94 => Some(Instruction::SUB(ArithmeticTarget::H)),
			0x95 => Some(Instruction::SUB(ArithmeticTarget::L)),
			0x96 => Some(Instruction::SUB(ArithmeticTarget::HLI)),
			0x97 => Some(Instruction::SUB(ArithmeticTarget::A)),
			// SBC (Subtract with Carry)
			0x98 => Some(Instruction::SBC(ArithmeticTarget::B)),
			0x99 => Some(Instruction::SBC(ArithmeticTarget::C)),
			0x9A => Some(Instruction::SBC(ArithmeticTarget::D)),
			0x9B => Some(Instruction::SBC(ArithmeticTarget::E)),
			0x9C => Some(Instruction::SBC(ArithmeticTarget::H)),
			0x9D => Some(Instruction::SBC(ArithmeticTarget::L)),
			0x9E => Some(Instruction::SBC(ArithmeticTarget::HLI)),
			0x9F => Some(Instruction::SBC(ArithmeticTarget::A)),
			// AND
			0xA0 => Some(Instruction::AND(ArithmeticTarget::B)),
			0xA1 => Some(Instruction::AND(ArithmeticTarget::C)),
			0xA2 => Some(Instruction::AND(ArithmeticTarget::D)),
			0xA3 => Some(Instruction::AND(ArithmeticTarget::E)),
			0xA4 => Some(Instruction::AND(ArithmeticTarget::H)),
			0xA5 => Some(Instruction::AND(ArithmeticTarget::L)),
			0xA6 => Some(Instruction::AND(ArithmeticTarget::HLI)),
			0xA7 => Some(Instruction::AND(ArithmeticTarget::A)),
			// XOR
			0xA8 => Some(Instruction::XOR(ArithmeticTarget::B)),
			0xA9 => Some(Instruction::XOR(ArithmeticTarget::C)),
			0xAA => Some(Instruction::XOR(ArithmeticTarget::D)),
			0xAB => Some(Instruction::XOR(ArithmeticTarget::E)),
			0xAC => Some(Instruction::XOR(ArithmeticTarget::H)),
			0xAD => Some(Instruction::XOR(ArithmeticTarget::L)),
			0xAE => Some(Instruction::XOR(ArithmeticTarget::HLI)),
			0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),
			// OR
			0xB0 => Some(Instruction::OR(ArithmeticTarget::B)),
			0xB1 => Some(Instruction::OR(ArithmeticTarget::C)),
			0xB2 => Some(Instruction::OR(ArithmeticTarget::D)),
			0xB3 => Some(Instruction::OR(ArithmeticTarget::E)),
			0xB4 => Some(Instruction::OR(ArithmeticTarget::H)),
			0xB5 => Some(Instruction::OR(ArithmeticTarget::L)),
			0xB6 => Some(Instruction::OR(ArithmeticTarget::HLI)),
			0xB7 => Some(Instruction::OR(ArithmeticTarget::A)),
			// CP (Compare)
			0xB8 => Some(Instruction::CP(ArithmeticTarget::B)),
			0xB9 => Some(Instruction::CP(ArithmeticTarget::C)),
			0xBA => Some(Instruction::CP(ArithmeticTarget::D)),
			0xBB => Some(Instruction::CP(ArithmeticTarget::E)),
			0xBC => Some(Instruction::CP(ArithmeticTarget::H)),
			0xBD => Some(Instruction::CP(ArithmeticTarget::L)),
			0xBE => Some(Instruction::CP(ArithmeticTarget::HLI)),
			0xBF => Some(Instruction::CP(ArithmeticTarget::A)),
			// Immediate 8-bit Arithmetic
			0xC6 => Some(Instruction::ADD(ArithmeticTarget::D8)),
			0xCE => Some(Instruction::ADC(ArithmeticTarget::D8)),
			0xD6 => Some(Instruction::SUB(ArithmeticTarget::D8)),
			0xDE => Some(Instruction::SBC(ArithmeticTarget::D8)),
			0xE6 => Some(Instruction::AND(ArithmeticTarget::D8)),
			0xEE => Some(Instruction::XOR(ArithmeticTarget::D8)),
			0xF6 => Some(Instruction::OR(ArithmeticTarget::D8)),
			0xFE => Some(Instruction::CP(ArithmeticTarget::D8)),
			// INC
			0x04 => Some(Instruction::INC(IncDecTarget::B)),
			0x0C => Some(Instruction::INC(IncDecTarget::C)),
			0x14 => Some(Instruction::INC(IncDecTarget::D)),
			0x1C => Some(Instruction::INC(IncDecTarget::E)),
			0x24 => Some(Instruction::INC(IncDecTarget::H)),
			0x2C => Some(Instruction::INC(IncDecTarget::L)),
			0x34 => Some(Instruction::INC(IncDecTarget::HLI)),
			0x3C => Some(Instruction::INC(IncDecTarget::A)),
			// DEC
			0x05 => Some(Instruction::DEC(IncDecTarget::B)),
			0x0D => Some(Instruction::DEC(IncDecTarget::C)),
			0x15 => Some(Instruction::DEC(IncDecTarget::D)),
			0x1D => Some(Instruction::DEC(IncDecTarget::E)),
			0x25 => Some(Instruction::DEC(IncDecTarget::H)),
			0x2D => Some(Instruction::DEC(IncDecTarget::L)),
			0x35 => Some(Instruction::DEC(IncDecTarget::HLI)),
			0x3D => Some(Instruction::DEC(IncDecTarget::A)),

			// Stack
			// POP
			0xC1 => Some(Instruction::POP(StackTarget::BC)),
			0xD1 => Some(Instruction::POP(StackTarget::DE)),
			0xE1 => Some(Instruction::POP(StackTarget::HL)),
			0xF1 => Some(Instruction::POP(StackTarget::AF)),
			// PUSH
			0xC5 => Some(Instruction::PUSH(StackTarget::BC)),
			0xD5 => Some(Instruction::PUSH(StackTarget::DE)),
			0xE5 => Some(Instruction::PUSH(StackTarget::HL)),
			0xF5 => Some(Instruction::PUSH(StackTarget::AF)),
			_ => None,
		}
	}
}

pub enum ArithmeticTarget {
	A,
	B,
	C,
	D,
	E,
	H,
	L,
	HLI,
	D8,
}

impl ArithmeticTarget {
	pub fn pc_increment(&self) -> u16 {
		match self {
			ArithmeticTarget::D8 => 2, // D8 instructions are 2 bytes long (opcode + value)
			_ => 1,                    // All other 8-bit targets are 1 byte long
		}
	}
}

pub enum IncDecTarget {
	A,
	B,
	C,
	D,
	E,
	H,
	L,
	HLI,
}

pub enum HLTarget {
	BC,
	DE,
	HL,
}

pub enum JumpTest {
	NotZero,
	Zero,
	NotCarry,
	Carry,
	Always,
}

pub enum LoadByteTarget {
	A,
	B,
	C,
	D,
	E,
	H,
	L,
	HLI,
}

pub enum LoadByteSource {
	A,
	B,
	C,
	D,
	E,
	H,
	L,
	D8,
	HLI,
}

pub enum LoadType {
	Byte(LoadByteTarget, LoadByteSource),
}

pub enum StackTarget {
	AF,
	BC,
	DE,
	HL,
}
