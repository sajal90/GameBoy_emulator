#[path = "flags_register.rs"]
mod flags_register;
#[path = "instruction.rs"]
mod instruction;
#[path = "memory.rs"]
mod memory;
#[path = "registers.rs"]
mod registers;

use flags_register::FlagsRegister;
use instruction::{ArithmeticTarget, HLTarget, Instruction, JumpTest};
use memory::MemoryBus;
use registers::Registers;

pub struct Cpu {
	registers: Registers,
	pc: u16,
	bus: MemoryBus,
}

impl Cpu {
	pub fn new() -> Self {
		let cpu = Cpu {
			registers: Registers::new(),
			pc: 0,
			bus: MemoryBus::new(),
		};
		cpu
	}

	fn step(&mut self) {
		let mut instruction_byte = self.bus.read_byte(self.pc);
		let prefixed = instruction_byte == 0xCB;
		if prefixed {
			instruction_byte = self.bus.read_byte(self.pc + 1);
		}

		let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
		{
			self.execute(instruction)
		} else {
			let description = format!(
				"0x{}{:x}",
				if prefixed { "cb" } else { "" },
				instruction_byte
			);
			panic!("Unkown instruction found for: {}", description)
		};

		self.pc = next_pc;
	}

	fn add(&mut self, val: u8) -> u8 {
		let (new_val, overflow) = self.registers.a.overflowing_add(val);
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = false;
		self.registers.f.carry = overflow;
		self.registers.f.half_carry = ((self.registers.a & 0xF) + (val & 0xF)) > 0xF;
		new_val
	}

	fn add_hl(&mut self, val: u16) {
		let hl = self.registers.get_hl();
		let (new_val, overflow) = hl.overflowing_add(val);
		self.registers.f.subtract = false;
		self.registers.f.carry = overflow;
		self.registers.f.half_carry = ((hl & 0xFFF) + (val & 0xFFF)) > 0xFFF;
		self.registers.set_hl(new_val);
	}

	fn add_carry(&mut self, val: u8) -> u8 {
		let carry = if self.registers.f.carry { 1 } else { 0 };
		let sum = (val as u16) + (carry as u16);
		let (new_val, overflow) = self.registers.a.overflowing_add(sum as u8);
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = false;
		self.registers.f.carry = overflow || sum > 0xFF;
		self.registers.f.half_carry = ((self.registers.a & 0xF) + (val & 0xF) + carry) > 0xF;
		new_val
	}

	fn sub(&mut self, val: u8) -> u8 {
		let (new_val, borrow) = self.registers.a.overflowing_sub(val);
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = true;
		self.registers.f.carry = borrow;
		self.registers.f.half_carry = (self.registers.a & 0xF) < (val & 0xF);
		new_val
	}

	fn sub_carry(&mut self, val: u8) -> u8 {
		let carry = if self.registers.f.carry { 1 } else { 0 };
		let sum = (val as u16) + (carry as u16);
		let (new_val, borrow) = self.registers.a.overflowing_sub(sum as u8);
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = true;
		self.registers.f.carry = borrow || sum > 0xFF;
		self.registers.f.half_carry = (self.registers.a & 0xF) < ((val & 0xF) + carry);
		new_val
	}

	fn and(&mut self, val: u8) -> u8 {
		let new_val = self.registers.a & val;
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = false;
		self.registers.f.carry = false;
		self.registers.f.half_carry = true;
		new_val
	}

	fn or(&mut self, val: u8) -> u8 {
		let new_val = self.registers.a | val;
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = false;
		self.registers.f.carry = false;
		self.registers.f.half_carry = false;
		new_val
	}

	fn xor(&mut self, val: u8) -> u8 {
		let new_val = self.registers.a ^ val;
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = false;
		self.registers.f.carry = false;
		self.registers.f.half_carry = false;
		new_val
	}

	fn jump(&mut self, should_jump: bool) -> u16 {
		if should_jump {
			let low = self.bus.read_byte(self.pc.wrapping_add(1)) as u16;
			let high = self.bus.read_byte(self.pc.wrapping_add(2)) as u16;
			(high << 8) | low
		} else {
			self.pc.wrapping_add(3)
		}
	}

	fn read_target_value(&self, target: &ArithmeticTarget) -> u8 {
		match target {
			ArithmeticTarget::A => self.registers.a,
			ArithmeticTarget::B => self.registers.b,
			ArithmeticTarget::C => self.registers.c,
			ArithmeticTarget::D => self.registers.d,
			ArithmeticTarget::E => self.registers.e,
			ArithmeticTarget::H => self.registers.h,
			ArithmeticTarget::L => self.registers.l,
		}
	}

	pub fn execute(&mut self, instruction: Instruction) -> u16 {
		match instruction {
			Instruction::ADD(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.add(value);
				self.pc.wrapping_add(1)
			}
			Instruction::ADDHL(target) => {
				match target {
					HLTarget::BC => {
						self.add_hl(self.registers.get_bc());
					}
					HLTarget::DE => {
						self.add_hl(self.registers.get_de());
					}
					HLTarget::HL => {
						self.add_hl(self.registers.get_hl());
					}
				}
				self.pc.wrapping_add(1)
			}
			Instruction::ADC(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.add_carry(value);
				self.pc.wrapping_add(1)
			}
			Instruction::SUB(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.sub(value);
				self.pc.wrapping_add(1)
			}
			Instruction::SBC(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.sub_carry(value);
				self.pc.wrapping_add(1)
			}
			Instruction::AND(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.and(value);
				self.pc.wrapping_add(1)
			}
			Instruction::OR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.or(value);
				self.pc.wrapping_add(1)
			}
			Instruction::XOR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.xor(value);
				self.pc.wrapping_add(1)
			}
			Instruction::OR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.or(value);
				self.pc.wrapping_add(1)
			}
			Instruction::XOR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.xor(value);
				self.pc.wrapping_add(1)
			}
			Instruction::JP(test) => {
                let jump_condition = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true
                };
                self.jump(jump_condition)
            },
		}
	}
}
