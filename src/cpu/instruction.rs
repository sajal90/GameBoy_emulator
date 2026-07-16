pub enum Instruction {
	ADD(ArithmeticTarget),
	ADDHL(HLTarget),
	ADC(ArithmeticTarget),
	SUB(ArithmeticTarget),
	SBC(ArithmeticTarget),
	AND(ArithmeticTarget),
	OR(ArithmeticTarget),
	XOR(ArithmeticTarget),
	JP(JumpTest),
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
			0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
			0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
			0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
			0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
			0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
			0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
			0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),
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
    Always
}