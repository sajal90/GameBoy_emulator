#[path = "flags_register.rs"]
mod flags_register;
#[path = "instruction.rs"]
mod instruction;
#[path = "memory.rs"]
mod memory;
#[path = "registers.rs"]
mod registers;

use instruction::{ArithmeticTarget, HLTarget, Instruction, JumpTest};
use instruction::{IncDecTarget, StackTarget};
use instruction::{LoadByteSource, LoadByteTarget, LoadType};
use memory::MemoryBus;
use registers::Registers;

pub struct Cpu {
	registers: Registers,
	pc: u16,
	sp: u16,
	bus: MemoryBus,
}

impl Cpu {
	pub fn new() -> Self {
		let cpu = Cpu {
			registers: Registers::new(),
			pc: 0,
			sp: 0,
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

	fn read_next_byte(&self) -> u8 {
		self.bus.read_byte(self.pc.wrapping_add(1))
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

	fn inc(&mut self, value: u8) -> u8 {
		let new_val = value.wrapping_add(1);
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = false;
		self.registers.f.half_carry = (value & 0xF) == 0xF;
		new_val
	}

	fn dec(&mut self, value: u8) -> u8 {
		let new_val = value.wrapping_sub(1);
		self.registers.f.zero = new_val == 0;
		self.registers.f.subtract = true;
		self.registers.f.half_carry = (value & 0xF) == 0;
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

	fn push(&mut self, value: u16) {
		self.sp = self.sp.wrapping_sub(1);
		self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

		self.sp = self.sp.wrapping_sub(1);
		self.bus.write_byte(self.sp, (value & 0xFF) as u8);
	}

	fn pop(&mut self) -> u16 {
		let lsb = self.bus.read_byte(self.sp) as u16;
		self.sp = self.sp.wrapping_add(1);

		let msb = self.bus.read_byte(self.sp) as u16;
		self.sp = self.sp.wrapping_add(1);

		(msb << 8) | lsb
	}

	fn read_next_word(&self) -> u16 {
		// Gameboy is little endian so read pc + 2 as most significant byte
		// and pc + 1 as least significant byte
		let lsb = self.bus.read_byte(self.pc.wrapping_add(1)) as u16;
		let msb = self.bus.read_byte(self.pc.wrapping_add(2)) as u16;
		(msb << 8) | lsb
	}

	fn call(&mut self, should_jump: bool) -> u16 {
		let next_pc = self.pc.wrapping_add(3);
		if should_jump {
			self.push(next_pc);
			self.read_next_word()
		} else {
			next_pc
		}
	}

	fn return_(&mut self, should_jump: bool) -> u16 {
		if should_jump {
			self.pop()
		} else {
			// RET conditional instructions are only 1 byte long!
			self.pc.wrapping_add(1)
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
			ArithmeticTarget::HLI => self.bus.read_byte(self.registers.get_hl()),
			ArithmeticTarget::D8 => self.read_next_byte(),
		}
	}

	pub fn execute(&mut self, instruction: Instruction) -> u16 {
		match instruction {
			Instruction::ADD(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.add(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::ADC(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.add_carry(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::SUB(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.sub(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::SBC(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.sub_carry(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::AND(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.and(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::OR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.or(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::XOR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.xor(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::OR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.or(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::XOR(target) => {
				let value = self.read_target_value(&target);
				self.registers.a = self.xor(value);
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::CP(target) => {
				let value = self.read_target_value(&target);
				self.sub(value); // Calls sub to set flags, but we ignore the returned value
				self.pc.wrapping_add(target.pc_increment())
			}
			Instruction::INC(target) => {
				match target {
					IncDecTarget::A => {
						self.registers.a = self.inc(self.registers.a);
					}
					IncDecTarget::B => {
						self.registers.b = self.inc(self.registers.b);
					}
					IncDecTarget::C => {
						self.registers.c = self.inc(self.registers.c);
					}
					IncDecTarget::D => {
						self.registers.d = self.inc(self.registers.d);
					}
					IncDecTarget::E => {
						self.registers.e = self.inc(self.registers.e);
					}
					IncDecTarget::H => {
						self.registers.h = self.inc(self.registers.h);
					}
					IncDecTarget::L => {
						self.registers.l = self.inc(self.registers.l);
					}
					IncDecTarget::HLI => {
						let addr = self.registers.get_hl();
						let value = self.bus.read_byte(addr);
						let new_val = self.inc(value);
						self.bus.write_byte(addr, new_val);
					}
				}
				self.pc.wrapping_add(1)
			}
			Instruction::DEC(target) => {
				match target {
					IncDecTarget::A => {
						self.registers.a = self.dec(self.registers.a);
					}
					IncDecTarget::B => {
						self.registers.b = self.dec(self.registers.b);
					}
					IncDecTarget::C => {
						self.registers.c = self.dec(self.registers.c);
					}
					IncDecTarget::D => {
						self.registers.d = self.dec(self.registers.d);
					}
					IncDecTarget::E => {
						self.registers.e = self.dec(self.registers.e);
					}
					IncDecTarget::H => {
						self.registers.h = self.dec(self.registers.h);
					}
					IncDecTarget::L => {
						self.registers.l = self.dec(self.registers.l);
					}
					IncDecTarget::HLI => {
						let addr = self.registers.get_hl();
						let value = self.bus.read_byte(addr);
						let new_val = self.dec(value);
						self.bus.write_byte(addr, new_val);
					}
				}
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
			Instruction::JP(test) => {
				let jump_condition = match test {
					JumpTest::NotZero => !self.registers.f.zero,
					JumpTest::NotCarry => !self.registers.f.carry,
					JumpTest::Zero => self.registers.f.zero,
					JumpTest::Carry => self.registers.f.carry,
					JumpTest::Always => true,
				};
				self.jump(jump_condition)
			}
			Instruction::LD(load_type) => match load_type {
				LoadType::Byte(target, source) => {
					let source_value = match source {
						LoadByteSource::A => self.registers.a,
						LoadByteSource::B => self.registers.b,
						LoadByteSource::C => self.registers.c,
						LoadByteSource::D => self.registers.d,
						LoadByteSource::E => self.registers.e,
						LoadByteSource::H => self.registers.h,
						LoadByteSource::L => self.registers.l,
						LoadByteSource::D8 => self.read_next_byte(),
						LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
					};

					match target {
						LoadByteTarget::A => self.registers.a = source_value,
						LoadByteTarget::B => self.registers.b = source_value,
						LoadByteTarget::C => self.registers.c = source_value,
						LoadByteTarget::D => self.registers.d = source_value,
						LoadByteTarget::E => self.registers.e = source_value,
						LoadByteTarget::H => self.registers.h = source_value,
						LoadByteTarget::L => self.registers.l = source_value,
						LoadByteTarget::HLI => {
							self.bus.write_byte(self.registers.get_hl(), source_value)
						}
					};

					match source {
						LoadByteSource::D8 => self.pc.wrapping_add(2),
						_ => self.pc.wrapping_add(1),
					}
				}
			},
			Instruction::PUSH(target) => {
				let value = match target {
					StackTarget::AF => self.registers.get_af(),
					StackTarget::BC => self.registers.get_bc(),
					StackTarget::DE => self.registers.get_de(),
					StackTarget::HL => self.registers.get_hl(),
				};
				self.push(value);
				self.pc.wrapping_add(1)
			}

			Instruction::POP(target) => {
				let result = self.pop();
				match target {
					StackTarget::AF => self.registers.set_af(result),
					StackTarget::BC => self.registers.set_bc(result),
					StackTarget::DE => self.registers.set_de(result),
					StackTarget::HL => self.registers.set_hl(result),
				};
				self.pc.wrapping_add(1)
			}
			Instruction::CALL(test) => {
				let jump_condition = match test {
					JumpTest::NotZero => !self.registers.f.zero,
					JumpTest::NotCarry => !self.registers.f.carry,
					JumpTest::Zero => self.registers.f.zero,
					JumpTest::Carry => self.registers.f.carry,
					JumpTest::Always => true,
				};
				self.call(jump_condition)
			}

			Instruction::RET(test) => {
				let jump_condition = match test {
					JumpTest::NotZero => !self.registers.f.zero,
					JumpTest::NotCarry => !self.registers.f.carry,
					JumpTest::Zero => self.registers.f.zero,
					JumpTest::Carry => self.registers.f.carry,
					JumpTest::Always => true,
				};
				self.return_(jump_condition)
			}
		}
	}
}
