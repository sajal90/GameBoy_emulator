#[path = "flags_register.rs"] mod flags_register;
use flags_register::FlagsRegister;

pub struct Registers {
  pub a: u8,
  pub b: u8,
  pub c: u8,
  pub d: u8,
  pub e: u8,
  pub f: FlagsRegister,
  pub h: u8,
  pub l: u8,
}

impl Registers {
    pub fn new() -> Registers {
        let registers = Registers {
            a: 0,
            b: 0,
            c: 0, 
            d: 0,
            e: 0,
            f: FlagsRegister::new(),
            h: 0,
            l: 0
        };
        registers
    }


    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f.clone())) as u16
    }

    fn set_af(&mut self, val: u16) {
        self.a = ((val >> 8) & 0xFF) as u8;
        self.f = FlagsRegister::from((val & 0xFF) as u8);
    }

    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    fn set_bc(&mut self, val: u16) {
        self.b = ((val >> 8) & 0xFF) as u8;
        self.c = (val & 0xFF) as u8;
    }

    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    fn set_de(&mut self, val: u16) {
        self.d = ((val >> 8) & 0xFF) as u8;
        self.e = (val & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    fn set_hl(&mut self, val: u16) {
        self.h = ((val >> 8) & 0xFF) as u8;
        self.l = (val & 0xFF) as u8;
    }
}