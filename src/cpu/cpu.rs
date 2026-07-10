#[path = "registers.rs"] mod registers;
#[path = "flags_register.rs"] mod flags_register;

use registers::Registers;
use flags_register::FlagsRegister;

enum Instruction {
  ADD(ArithmeticTarget),
  ADDHL(HLTarget),
}

enum ArithmeticTarget {
  A, B, C, D, E, H, L,
}

enum HLTarget {
    BC, DE, HL,
}

pub struct Cpu {
    registers: Registers,
}

impl Cpu {
    pub fn new() -> Self {
        let cpu = Cpu {
            registers: Registers::new(),
        };
        cpu
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

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                match target {
                    ArithmeticTarget::A => { self.registers.a = self.add(self.registers.a); },
                    ArithmeticTarget::B => { self.registers.a = self.add(self.registers.b); },
                    ArithmeticTarget::C => { self.registers.a = self.add(self.registers.c); },
                    ArithmeticTarget::D => { self.registers.a = self.add(self.registers.d); },
                    ArithmeticTarget::E => { self.registers.a = self.add(self.registers.e); },
                    ArithmeticTarget::H => { self.registers.a = self.add(self.registers.h); },
                    ArithmeticTarget::L => { self.registers.a = self.add(self.registers.l); },
                }
            },
            Instruction::ADDHL(target) => {
                match target {
                    HLTarget::BC => { self.add_hl(self.registers.get_bc()); },
                    HLTarget::DE => { self.add_hl(self.registers.get_de()); },
                    HLTarget::HL => { self.add_hl(self.registers.get_hl()); },
                }
            },
            _ => {},
        }
    }
}