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
	INC16(IncDecTarget16),
	DEC16(IncDecTarget16),
	// Flag and accumulator operations
	RLCA,
	RRCA,
	RLA,
	RRA,
	DAA,
	CPL,
	SCF,
	CCF,
	// jump instructions
	JP(JumpTest),
	// load instructions
	LD(LoadType),
	// stack instructions
	PUSH(StackTarget),
	POP(StackTarget),
	CALL(JumpTest),
	RET(JumpTest),

	JR(JumpTest),
	RST(u16), // The u16 is the hardcoded address to call
	RETI,
	NOP,
	HALT,
	DI, // Disable Interrupts
	EI, // Enable Interrupts
	STOP,
	JPHL,
	ADDSP,
	LDHLSP,
	// Prefix Instructions
	RLC(PrefixTarget),
	RRC(PrefixTarget),
	RL(PrefixTarget),
	RR(PrefixTarget),
	SLA(PrefixTarget),
	SRA(PrefixTarget),
	SWAP(PrefixTarget),
	SRL(PrefixTarget),
	BIT(u8, PrefixTarget),
	RES(u8, PrefixTarget),
	SET(u8, PrefixTarget),
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

pub enum IncDecTarget16 {
	BC,
	DE,
	HL,
	SP,
}

pub enum HLTarget {
	BC,
	DE,
	HL,
	SP,
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
	BCI,
	DEI,
	HLIPlus,
	HLIMinus,
	HighC,  // (0xFF00 + C)
	HighD8, // (0xFF00 + d8)
	A16I,   // (a16)
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
	BCI,
	DEI,
	HLIPlus,
	HLIMinus,
	HighC,
	HighD8,
	A16I,
}

pub enum LoadType {
	Byte(LoadByteTarget, LoadByteSource),
	Word(LoadWordTarget, LoadWordSource),
}

pub enum LoadWordTarget {
	BC,
	DE,
	HL,
	SP,
	A16I,
}

pub enum LoadWordSource {
	D16,
	SP,
	HL,
}

pub enum StackTarget {
	AF,
	BC,
	DE,
	HL,
}

pub enum PrefixTarget {
	B,
	C,
	D,
	E,
	H,
	L,
	HLI,
	A,
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
		// Extract the lowest 3 bits to determine the target register
		let target = match byte & 0x07 {
			0 => PrefixTarget::B,
			1 => PrefixTarget::C,
			2 => PrefixTarget::D,
			3 => PrefixTarget::E,
			4 => PrefixTarget::H,
			5 => PrefixTarget::L,
			6 => PrefixTarget::HLI,
			7 => PrefixTarget::A,
			_ => unreachable!(), // 0x07 mask guarantees 0-7
		};

		// Use the remaining bits to determine the instruction
		match byte {
			0x00..=0x3F => {
				let operation = byte >> 3;
				match operation {
					0 => Some(Instruction::RLC(target)),
					1 => Some(Instruction::RRC(target)),
					2 => Some(Instruction::RL(target)),
					3 => Some(Instruction::RR(target)),
					4 => Some(Instruction::SLA(target)),
					5 => Some(Instruction::SRA(target)),
					6 => Some(Instruction::SWAP(target)),
					7 => Some(Instruction::SRL(target)),
					_ => unreachable!(),
				}
			}
			// BIT ops
			0x40..=0x7F => {
				let bit = (byte >> 3) & 0x07;
				Some(Instruction::BIT(bit, target))
			}
			// RES ops
			0x80..=0xBF => {
				let bit = (byte >> 3) & 0x07;
				Some(Instruction::RES(bit, target))
			}
			// SET ops
			0xC0..=0xFF => {
				let bit = (byte >> 3) & 0x07;
				Some(Instruction::SET(bit, target))
			}
			_ => None,
		}
	}

	fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
		match byte {
			// 245 standard opcodes, remaining 11 falls on _, as they
			// are illegal opcodes

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

			// 16-bit Arithmetic
			// 16-bit ADD HL
			0x09 => Some(Instruction::ADDHL(HLTarget::BC)),
			0x19 => Some(Instruction::ADDHL(HLTarget::DE)),
			0x29 => Some(Instruction::ADDHL(HLTarget::HL)),
			0x39 => Some(Instruction::ADDHL(HLTarget::SP)),
			// 16-bit INC/DEC
			0x03 => Some(Instruction::INC16(IncDecTarget16::BC)),
			0x13 => Some(Instruction::INC16(IncDecTarget16::DE)),
			0x23 => Some(Instruction::INC16(IncDecTarget16::HL)),
			0x33 => Some(Instruction::INC16(IncDecTarget16::SP)),

			0x0B => Some(Instruction::DEC16(IncDecTarget16::BC)),
			0x1B => Some(Instruction::DEC16(IncDecTarget16::DE)),
			0x2B => Some(Instruction::DEC16(IncDecTarget16::HL)),
			0x3B => Some(Instruction::DEC16(IncDecTarget16::SP)),

			// Accumulator & Flag Specials
			0x07 => Some(Instruction::RLCA),
			0x0F => Some(Instruction::RRCA),
			0x17 => Some(Instruction::RLA),
			0x1F => Some(Instruction::RRA),
			0x27 => Some(Instruction::DAA),
			0x2F => Some(Instruction::CPL),
			0x37 => Some(Instruction::SCF),
			0x3F => Some(Instruction::CCF),

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
			// CALL
			0xCD => Some(Instruction::CALL(JumpTest::Always)),
			0xC4 => Some(Instruction::CALL(JumpTest::NotZero)),
			0xCC => Some(Instruction::CALL(JumpTest::Zero)),
			0xD4 => Some(Instruction::CALL(JumpTest::NotCarry)),
			0xDC => Some(Instruction::CALL(JumpTest::Carry)),
			// RET
			0xC9 => Some(Instruction::RET(JumpTest::Always)),
			0xC0 => Some(Instruction::RET(JumpTest::NotZero)),
			0xC8 => Some(Instruction::RET(JumpTest::Zero)),
			0xD0 => Some(Instruction::RET(JumpTest::NotCarry)),
			0xD8 => Some(Instruction::RET(JumpTest::Carry)),

			// JUMPS
			0xC3 => Some(Instruction::JP(JumpTest::Always)),
			0xC2 => Some(Instruction::JP(JumpTest::NotZero)),
			0xCA => Some(Instruction::JP(JumpTest::Zero)),
			0xD2 => Some(Instruction::JP(JumpTest::NotCarry)),
			0xDA => Some(Instruction::JP(JumpTest::Carry)),
			// NOP
			0x00 => Some(Instruction::NOP),

			// Interrupts
			0xF3 => Some(Instruction::DI),
			0xFB => Some(Instruction::EI),
			0xD9 => Some(Instruction::RETI),

			// JR (Jump Relative)
			0x18 => Some(Instruction::JR(JumpTest::Always)),
			0x20 => Some(Instruction::JR(JumpTest::NotZero)),
			0x28 => Some(Instruction::JR(JumpTest::Zero)),
			0x30 => Some(Instruction::JR(JumpTest::NotCarry)),
			0x38 => Some(Instruction::JR(JumpTest::Carry)),

			// RST (Restart)
			0xC7 => Some(Instruction::RST(0x0000)),
			0xCF => Some(Instruction::RST(0x0008)),
			0xD7 => Some(Instruction::RST(0x0010)),
			0xDF => Some(Instruction::RST(0x0018)),
			0xE7 => Some(Instruction::RST(0x0020)),
			0xEF => Some(Instruction::RST(0x0028)),
			0xF7 => Some(Instruction::RST(0x0030)),
			0xFF => Some(Instruction::RST(0x0038)),

			// 8-BIT REGISTER-TO-REGISTER LOADS
			// LD B, r
			0x40 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::B,
			))),
			0x41 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::C,
			))),
			0x42 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::D,
			))),
			0x43 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::E,
			))),
			0x44 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::H,
			))),
			0x45 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::L,
			))),
			0x46 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::HLI,
			))),
			0x47 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::A,
			))),

			// LD C, r
			0x48 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::B,
			))),
			0x49 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::C,
			))),
			0x4A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::D,
			))),
			0x4B => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::E,
			))),
			0x4C => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::H,
			))),
			0x4D => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::L,
			))),
			0x4E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::HLI,
			))),
			0x4F => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::A,
			))),

			// LD D, r
			0x50 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::B,
			))),
			0x51 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::C,
			))),
			0x52 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::D,
			))),
			0x53 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::E,
			))),
			0x54 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::H,
			))),
			0x55 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::L,
			))),
			0x56 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::HLI,
			))),
			0x57 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::A,
			))),

			// LD E, r
			0x58 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::B,
			))),
			0x59 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::C,
			))),
			0x5A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::D,
			))),
			0x5B => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::E,
			))),
			0x5C => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::H,
			))),
			0x5D => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::L,
			))),
			0x5E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::HLI,
			))),
			0x5F => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::A,
			))),

			// LD H, r
			0x60 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::B,
			))),
			0x61 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::C,
			))),
			0x62 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::D,
			))),
			0x63 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::E,
			))),
			0x64 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::H,
			))),
			0x65 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::L,
			))),
			0x66 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::HLI,
			))),
			0x67 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::A,
			))),

			// LD L, r
			0x68 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::B,
			))),
			0x69 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::C,
			))),
			0x6A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::D,
			))),
			0x6B => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::E,
			))),
			0x6C => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::H,
			))),
			0x6D => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::L,
			))),
			0x6E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::HLI,
			))),
			0x6F => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::A,
			))),

			// LD (HL), r
			0x70 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::B,
			))),
			0x71 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::C,
			))),
			0x72 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::D,
			))),
			0x73 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::E,
			))),
			0x74 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::H,
			))),
			0x75 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::L,
			))),
			0x76 => Some(Instruction::HALT), // HALT instruction is at 0x76, not a load instruction
			0x77 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::A,
			))),

			// LD A, r
			0x78 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::B,
			))),
			0x79 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::C,
			))),
			0x7A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::D,
			))),
			0x7B => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::E,
			))),
			0x7C => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::H,
			))),
			0x7D => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::L,
			))),
			0x7E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::HLI,
			))),
			0x7F => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::A,
			))),

			// 16-bit Load
			0x01 => Some(Instruction::LD(LoadType::Word(
				LoadWordTarget::BC,
				LoadWordSource::D16,
			))),
			0x11 => Some(Instruction::LD(LoadType::Word(
				LoadWordTarget::DE,
				LoadWordSource::D16,
			))),
			0x21 => Some(Instruction::LD(LoadType::Word(
				LoadWordTarget::HL,
				LoadWordSource::D16,
			))),
			0x31 => Some(Instruction::LD(LoadType::Word(
				LoadWordTarget::SP,
				LoadWordSource::D16,
			))),
			// Memory Pointers
			0x02 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::BCI,
				LoadByteSource::A,
			))),
			0x0A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::BCI,
			))),
			0x12 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::DEI,
				LoadByteSource::A,
			))),
			0x1A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::DEI,
			))),

			// Auto-Increment/Decrement
			0x22 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLIPlus,
				LoadByteSource::A,
			))),
			0x2A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::HLIPlus,
			))),
			0x32 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLIMinus,
				LoadByteSource::A,
			))),
			0x3A => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::HLIMinus,
			))),

			// High RAM (Zero Page)
			0xE0 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HighD8,
				LoadByteSource::A,
			))),
			0xF0 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::HighD8,
			))),
			0xE2 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HighC,
				LoadByteSource::A,
			))),
			0xF2 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::HighC,
			))),

			// Absolute 16-bit Loads
			0xEA => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A16I,
				LoadByteSource::A,
			))),
			0xFA => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::A16I,
			))),
			0x08 => Some(Instruction::LD(LoadType::Word(
				LoadWordTarget::A16I,
				LoadWordSource::SP,
			))),

			// Immediate 8-bit Register Loads (LD r, d8)
			0x06 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::B,
				LoadByteSource::D8,
			))),
			0x0E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::C,
				LoadByteSource::D8,
			))),
			0x16 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::D,
				LoadByteSource::D8,
			))),
			0x1E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::E,
				LoadByteSource::D8,
			))),
			0x26 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::H,
				LoadByteSource::D8,
			))),
			0x2E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::L,
				LoadByteSource::D8,
			))),
			0x36 => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::HLI,
				LoadByteSource::D8,
			))),
			0x3E => Some(Instruction::LD(LoadType::Byte(
				LoadByteTarget::A,
				LoadByteSource::D8,
			))),
			0x10 => Some(Instruction::STOP),
			0xF9 => Some(Instruction::LD(LoadType::Word(
				LoadWordTarget::SP,
				LoadWordSource::HL,
			))),
			0xE9 => Some(Instruction::JPHL),
			0xE8 => Some(Instruction::ADDSP),
			0xF8 => Some(Instruction::LDHLSP),
			_ => None,
		}
	}
}
