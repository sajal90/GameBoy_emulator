const RAM_SIZE = 1 << 13;
const VRAM_SIZE = 1 << 13;


struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool
}

struct FlagsRegister {
   zero: bool,
   subtract: bool,
   half_carry: bool,
   carry: bool
}
const ZERO_FLAG_POSITION: u8 = 7;
const SUBTRACT_FLAG_POSITION: u8 = 6;
const HALF_CARRY_FLAG_POSITION: u8 = 5;
const CARRY_FLAG_POSITION: u8 = 4;

impl std::convert::From<FlagsRegister> for u8  {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero       { 1 } else { 0 }) << ZERO_FLAG_POSITION |
        (if flag.subtract   { 1 } else { 0 }) << SUBTRACT_FLAG_POSITION |
        (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_POSITION |
        (if flag.carry      { 1 } else { 0 }) << CARRY_FLAG_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry
        }
    }
}

struct Registers {
  a: u8,
  b: u8,
  c: u8,
  d: u8,
  e: u8,
  f: FlagsRegister,
  h: u8,
  l: u8,
}

impl Registers {
    fn get_af(&self) -> u16 {
        (self.a as u16 << 8) | self.f as u16;
    }

    fn set_af(&mut self, val: u16) {
        self.a = ((val >> 8) & 0xFF) as u8;
        self.f = (val & 0xFF) as u8;
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16 << 8) | self.c as u16;
    }

    fn set_bc(&mut self, val: u16) {
        self.b = ((val >> 8) & 0xFF) as u8;
        self.c = (val & 0xFF) as u8;
    }

    fn get_de(&self) -> u16 {
        (self.d as u16 << 8) | self.e as u16;
    }

    fn set_de(&mut self, val: u16) {
        self.d = ((val >> 8) & 0xFF) as u8;
        self.e = (val & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16 << 8) | self.l as u16;
    }

    fn set_hl(&mut self, val: u16) {
        self.h = ((val >> 8) & 0xFF) as u8;
        self.l = (val & 0xFF) as u8;
    }
}

struct Cpu {
};

impl Cpu {
    pub fn new() -> Self {
    }

    pub fn fetch() -> Self {
    }
}
