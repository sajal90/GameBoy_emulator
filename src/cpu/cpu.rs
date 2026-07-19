#[path = "flags_register.rs"]
mod flags_register;
#[path = "instruction.rs"]
mod instruction;
#[path = "memory.rs"]
mod memory;
#[path = "registers.rs"]
mod registers;

use instruction::{ArithmeticTarget, HLTarget, Instruction, JumpTest, PrefixTarget};
use instruction::{IncDecTarget, IncDecTarget16, StackTarget};
use instruction::{LoadByteSource, LoadByteTarget, LoadType, LoadWordSource, LoadWordTarget};
use memory::MemoryBus;
use registers::Registers;

pub struct Cpu {
	registers: Registers,
	pc: u16,
	sp: u16,
	bus: MemoryBus,
	interrupt_master_enable: bool,
	halted: bool,
}

impl Cpu {
	pub fn new() -> Self {
		let cpu = Cpu {
			registers: Registers::new(),
			// Start at the Cartridge Entry Point to skip the missing Boot ROM
			pc: 0x0100,
			// The standard value of the Stack Pointer after the Boot ROM finishes
			sp: 0xFFFE,
			bus: MemoryBus::new(),
			interrupt_master_enable: true,
			halted: false,
		};
		cpu
	}

	pub fn load_rom(&mut self, data: Vec<u8>) {
		self.bus.load_rom(data);
	}

	pub fn step(&mut self) {
		self.handle_interrupts();

		if self.halted {
			return;
		}

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

	fn handle_interrupts(&mut self) {
		if !self.interrupt_master_enable && !self.halted {
			return;
		}

		let interrupt_enable = self.bus.read_byte(0xFFFF);
		let interrupt_flag = self.bus.read_byte(0xFF0F);

		// Only the bottom 5 bits represent actual interrupts
		let pending = interrupt_enable & interrupt_flag & 0x1F;

		if pending != 0 {
			// Any pending interrupt wakes the CPU up, even if IME is false
			self.halted = false;

			if self.interrupt_master_enable {
				for bit in 0..5 {
					if (pending & (1 << bit)) != 0 {
						self.interrupt_master_enable = false;

						self.bus.write_byte(0xFF0F, interrupt_flag & !(1 << bit));

						self.push(self.pc);

						self.pc = match bit {
							0 => 0x0040, // VBlank
							1 => 0x0048, // LCD STAT
							2 => 0x0050, // Timer
							3 => 0x0058, // Serial
							4 => 0x0060, // Joypad
							_ => unreachable!(),
						};

						// break after handling the highest priority interrupt
						break;
					}
				}
			}
		}
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
			// RET conditional instructions are only 1 byte long
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

	fn read_prefix_target(&self, target: &PrefixTarget) -> u8 {
		match target {
			PrefixTarget::B => self.registers.b,
			PrefixTarget::C => self.registers.c,
			PrefixTarget::D => self.registers.d,
			PrefixTarget::E => self.registers.e,
			PrefixTarget::H => self.registers.h,
			PrefixTarget::L => self.registers.l,
			PrefixTarget::HLI => self.bus.read_byte(self.registers.get_hl()),
			PrefixTarget::A => self.registers.a,
		}
	}

	fn write_prefix_target(&mut self, target: &PrefixTarget, value: u8) {
		match target {
			PrefixTarget::B => self.registers.b = value,
			PrefixTarget::C => self.registers.c = value,
			PrefixTarget::D => self.registers.d = value,
			PrefixTarget::E => self.registers.e = value,
			PrefixTarget::H => self.registers.h = value,
			PrefixTarget::L => self.registers.l = value,
			PrefixTarget::HLI => self.bus.write_byte(self.registers.get_hl(), value),
			PrefixTarget::A => self.registers.a = value,
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
					HLTarget::SP => {
						self.add_hl(self.sp);
					}
				}
				self.pc.wrapping_add(1)
			}
			// 16-BIT INC/DEC
			Instruction::INC16(target) => {
				match target {
					IncDecTarget16::BC => self
						.registers
						.set_bc(self.registers.get_bc().wrapping_add(1)),
					IncDecTarget16::DE => self
						.registers
						.set_de(self.registers.get_de().wrapping_add(1)),
					IncDecTarget16::HL => self
						.registers
						.set_hl(self.registers.get_hl().wrapping_add(1)),
					IncDecTarget16::SP => self.sp = self.sp.wrapping_add(1),
				}
				self.pc.wrapping_add(1)
			}
			Instruction::DEC16(target) => {
				match target {
					IncDecTarget16::BC => self
						.registers
						.set_bc(self.registers.get_bc().wrapping_sub(1)),
					IncDecTarget16::DE => self
						.registers
						.set_de(self.registers.get_de().wrapping_sub(1)),
					IncDecTarget16::HL => self
						.registers
						.set_hl(self.registers.get_hl().wrapping_sub(1)),
					IncDecTarget16::SP => self.sp = self.sp.wrapping_sub(1),
				}
				self.pc.wrapping_add(1)
			}
			//  ACCUMULATOR/FLAG
			Instruction::RLCA => {
				let carry = (self.registers.a & 0b1000_0000) >> 7;
				self.registers.a = (self.registers.a << 1) | carry;
				self.registers.f.zero = false; // RLCA uniquely clears the zero flag
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;
				self.pc.wrapping_add(1)
			}
			Instruction::RRCA => {
				let carry = self.registers.a & 0b0000_0001;
				self.registers.a = (self.registers.a >> 1) | (carry << 7);
				self.registers.f.zero = false; // RRCA uniquely clears the zero flag
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;
				self.pc.wrapping_add(1)
			}
			Instruction::RLA => {
				let bit7 = (self.registers.a & 0b1000_0000) >> 7;
				let carry = if self.registers.f.carry { 1 } else { 0 };
				self.registers.a = (self.registers.a << 1) | carry;
				self.registers.f.zero = false;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = bit7 == 1;
				self.pc.wrapping_add(1)
			}
			Instruction::RRA => {
				let bit0 = self.registers.a & 0b0000_0001;
				let carry = if self.registers.f.carry { 1 } else { 0 };
				self.registers.a = (self.registers.a >> 1) | (carry << 7);
				self.registers.f.zero = false;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = bit0 == 1;
				self.pc.wrapping_add(1)
			}
			Instruction::CPL => {
				self.registers.a = !self.registers.a;
				self.registers.f.subtract = true;
				self.registers.f.half_carry = true;
				self.pc.wrapping_add(1)
			}
			Instruction::SCF => {
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = true;
				self.pc.wrapping_add(1)
			}
			Instruction::CCF => {
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = !self.registers.f.carry;
				self.pc.wrapping_add(1)
			}
			Instruction::DAA => {
				// Logic from https://github.com/libraries/gameboy/blob/17e56feafdd412d5c8e5fe1471f112ec4b40322c/src/cpu.rs#L169
				let mut a = self.registers.a;

				let mut adjust = if self.registers.f.carry { 0x60 } else { 0x00 };

				if self.registers.f.half_carry {
					adjust |= 0x06;
				}

				if !self.registers.f.subtract {
					if a & 0x0F > 0x09 {
						adjust |= 0x06;
					}
					if a > 0x99 {
						adjust |= 0x60;
					}
					a = a.wrapping_add(adjust);
				} else {
					a = a.wrapping_sub(adjust);
				}

				self.registers.f.carry = adjust >= 0x60;
				self.registers.f.half_carry = false;
				self.registers.f.zero = a == 0x00;
				self.registers.a = a;

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
					// Calculating PC increment before moving target/source
					let pc_inc = match (&target, &source) {
						(LoadByteTarget::A16I, _) | (_, LoadByteSource::A16I) => 3,
						(LoadByteTarget::HighD8, _) | (_, LoadByteSource::HighD8) => 2,
						(_, LoadByteSource::D8) => 2,
						_ => 1,
					};
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
						LoadByteSource::BCI => self.bus.read_byte(self.registers.get_bc()),
						LoadByteSource::DEI => self.bus.read_byte(self.registers.get_de()),
						LoadByteSource::HLIPlus => {
							let addr = self.registers.get_hl();
							self.registers.set_hl(addr.wrapping_add(1));
							self.bus.read_byte(addr)
						}
						LoadByteSource::HLIMinus => {
							let addr = self.registers.get_hl();
							self.registers.set_hl(addr.wrapping_sub(1));
							self.bus.read_byte(addr)
						}
						LoadByteSource::HighC => {
							self.bus.read_byte(0xFF00 | (self.registers.c as u16))
						}
						LoadByteSource::HighD8 => {
							self.bus.read_byte(0xFF00 | (self.read_next_byte() as u16))
						}
						LoadByteSource::A16I => self.bus.read_byte(self.read_next_word()),
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
						LoadByteTarget::BCI => {
							self.bus.write_byte(self.registers.get_bc(), source_value)
						}
						LoadByteTarget::DEI => {
							self.bus.write_byte(self.registers.get_de(), source_value)
						}
						LoadByteTarget::HLIPlus => {
							let addr = self.registers.get_hl();
							self.bus.write_byte(addr, source_value);
							self.registers.set_hl(addr.wrapping_add(1));
						}
						LoadByteTarget::HLIMinus => {
							let addr = self.registers.get_hl();
							self.bus.write_byte(addr, source_value);
							self.registers.set_hl(addr.wrapping_sub(1));
						}
						LoadByteTarget::HighC => self
							.bus
							.write_byte(0xFF00 | (self.registers.c as u16), source_value),
						LoadByteTarget::HighD8 => {
							let offset = self.read_next_byte() as u16;
							self.bus.write_byte(0xFF00 | offset, source_value);
						}
						LoadByteTarget::A16I => {
							self.bus.write_byte(self.read_next_word(), source_value)
						}
					};

					self.pc.wrapping_add(pc_inc)
				}
				LoadType::Word(target, source) => {
					let pc_inc = match (&target, &source) {
						(LoadWordTarget::A16I, _) => 3,
						(_, LoadWordSource::D16) => 3,
						_ => 1,
					};

					let source_value = match source {
						LoadWordSource::D16 => self.read_next_word(),
						LoadWordSource::SP => self.sp,
						LoadWordSource::HL => self.registers.get_hl(),
					};

					match target {
						LoadWordTarget::BC => self.registers.set_bc(source_value),
						LoadWordTarget::DE => self.registers.set_de(source_value),
						LoadWordTarget::HL => self.registers.set_hl(source_value),
						LoadWordTarget::SP => self.sp = source_value,
						LoadWordTarget::A16I => {
							let addr = self.read_next_word();
							self.bus.write_byte(addr, (source_value & 0xFF) as u8);
							self.bus
								.write_byte(addr.wrapping_add(1), (source_value >> 8) as u8);
						}
					};
					self.pc.wrapping_add(pc_inc)
				}
			},
			Instruction::STOP => {
				// stubbing this by skipping 2 bytes. Original games rarely use STOP
				// due to hardware glitches, and it naturally swallows the following byte.
				// only need to implement the real logic later if I add Game Boy Color speed switching.
				self.pc.wrapping_add(2)
			}

			Instruction::JPHL => self.registers.get_hl(),

			Instruction::ADDSP => {
				let offset = self.read_next_byte();
				let sp = self.sp;

				self.registers.f.zero = false;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = (sp & 0x0F) + ((offset as u16) & 0x0F) > 0x0F;
				self.registers.f.carry = (sp & 0xFF) + ((offset as u16) & 0xFF) > 0xFF;

				// Cast the offset to a signed i8, then sign-extend to u16 for actual math
				self.sp = sp.wrapping_add((offset as i8 as i16) as u16);

				self.pc.wrapping_add(2) // 1 byte opcode + 1 byte offset
			}

			Instruction::LDHLSP => {
				let offset = self.read_next_byte();
				let sp = self.sp;

				self.registers.f.zero = false;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = (sp & 0x0F) + ((offset as u16) & 0x0F) > 0x0F;
				self.registers.f.carry = (sp & 0xFF) + ((offset as u16) & 0xFF) > 0xFF;

				let result = sp.wrapping_add((offset as i8 as i16) as u16);
				self.registers.set_hl(result);

				self.pc.wrapping_add(2)
			}
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
			Instruction::NOP => self.pc.wrapping_add(1),
			Instruction::DI => {
				self.interrupt_master_enable = false;
				self.pc.wrapping_add(1)
			}
			Instruction::EI => {
				self.interrupt_master_enable = true;
				// On real hardware, EI delays enabling interrupts by 1 instruction
				// ignoring the delay for now
				self.pc.wrapping_add(1)
			}
			Instruction::RETI => {
				self.interrupt_master_enable = true;
				self.return_(true)
			}
			Instruction::HALT => {
				self.halted = true;
				self.pc.wrapping_add(1)
			}
			Instruction::RST(address) => {
				let next_pc = self.pc.wrapping_add(1);
				self.push(next_pc);
				address
			}
			Instruction::JR(test) => {
				let jump_condition = match test {
					JumpTest::NotZero => !self.registers.f.zero,
					JumpTest::NotCarry => !self.registers.f.carry,
					JumpTest::Zero => self.registers.f.zero,
					JumpTest::Carry => self.registers.f.carry,
					JumpTest::Always => true,
				};

				let next_pc = self.pc.wrapping_add(2); // JR is a 2-byte instruction

				if jump_condition {
					let offset = self.bus.read_byte(self.pc.wrapping_add(1)) as i8;
					next_pc.wrapping_add(offset as u16)
				} else {
					next_pc
				}
			}

			// PREFIX INSTRUCTIONS
			Instruction::RLC(target) => {
				let value = self.read_prefix_target(&target);
				let carry = (value & 0x80) >> 7;
				let result = (value << 1) | carry;

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::RRC(target) => {
				let value = self.read_prefix_target(&target);
				let carry = value & 0x01;
				let result = (value >> 1) | (carry << 7);

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::RL(target) => {
				let value = self.read_prefix_target(&target);
				let carry = if self.registers.f.carry { 1 } else { 0 };
				let new_carry = (value & 0x80) >> 7;
				let result = (value << 1) | carry;

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = new_carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::RR(target) => {
				let value = self.read_prefix_target(&target);
				let carry = if self.registers.f.carry { 1 } else { 0 };
				let new_carry = value & 0x01;
				let result = (value >> 1) | (carry << 7);

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = new_carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::SLA(target) => {
				let value = self.read_prefix_target(&target);
				let carry = (value & 0x80) >> 7;
				let result = value << 1;

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::SRA(target) => {
				let value = self.read_prefix_target(&target);
				let carry = value & 0x01;
				let result = (value >> 1) | (value & 0x80); // Bit 7 stays the same

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::SWAP(target) => {
				let value = self.read_prefix_target(&target);
				let result = (value << 4) | (value >> 4);

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = false;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::SRL(target) => {
				let value = self.read_prefix_target(&target);
				let carry = value & 0x01;
				let result = value >> 1;

				self.registers.f.zero = result == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = false;
				self.registers.f.carry = carry == 1;

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::BIT(bit, target) => {
				let value = self.read_prefix_target(&target);

				self.registers.f.zero = (value & (1 << bit)) == 0;
				self.registers.f.subtract = false;
				self.registers.f.half_carry = true;
				// The carry flag is untouched by the BIT instruction

				self.pc.wrapping_add(2)
			}
			Instruction::RES(bit, target) => {
				let value = self.read_prefix_target(&target);
				let result = value & !(1 << bit);

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
			Instruction::SET(bit, target) => {
				let value = self.read_prefix_target(&target);
				let result = value | (1 << bit);

				self.write_prefix_target(&target, result);
				self.pc.wrapping_add(2)
			}
		}
	}
}
