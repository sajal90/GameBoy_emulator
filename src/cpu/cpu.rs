#[path = "registers.rs"] mod registers;
#[path = "flags_register.rs"] mod flags_register;
#[path = "memory.rs"] mod memory;
#[path = "instruction.rs"] mod instruction;

use registers::Registers;
use flags_register::FlagsRegister;
use memory::MemoryBus;
use instruction::{Instruction, ArithmeticTarget, HLTarget};

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
        // BUG: if val == 0xFF and carry is set, this will overflow
        //      same in the sub_carry method
        let (new_val, overflow) = self.registers.a.overflowing_add(val + carry);
        self.registers.f.zero = new_val == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = overflow;
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
        let (new_val, borrow) = self.registers.a.overflowing_sub(val + carry);
        self.registers.f.zero = new_val == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = borrow;
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

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.add(value);
            },
            Instruction::ADDHL(target) => {
                match target {
                    HLTarget::BC => { self.add_hl(self.registers.get_bc()); },
                    HLTarget::DE => { self.add_hl(self.registers.get_de()); },
                    HLTarget::HL => { self.add_hl(self.registers.get_hl()); },
                }
            },
            Instruction::ADC(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.add_carry(value);
            },
            Instruction::SUB(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.sub(value);
            },
            Instruction::SBC(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.sub_carry(value);
            },

            Instruction::AND(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.and(value);
            },
            Instruction::OR(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.or(value);
            },
            Instruction::XOR(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.xor(value);
            },
            Instruction::OR(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.or(value);
            },
            
            Instruction::XOR(target) => {
                let value = self.read_target_value(&target);
                self.registers.a = self.xor(value);
            },
        }
    }
}