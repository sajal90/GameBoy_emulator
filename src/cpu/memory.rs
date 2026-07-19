pub struct MemoryBus {
    rom: Vec<u8>,
    vram: [u8; 0x2000],
    eram: [u8; 0x2000],
    wram: [u8; 0x2000],
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    hram: [u8; 0x7F],
    interrupt_enable: u8,
}

impl MemoryBus {
    pub fn new() -> Self {
        MemoryBus {
            // Initialize with a blank 32KB ROM so it doesn't crash before loading
            rom: vec![0; 0x8000], 
            vram: [0; 0x2000],
            eram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            interrupt_enable: 0,
        }
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        self.rom = data;
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom[address as usize],
            
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            
            0xA000..=0xBFFF => self.eram[(address - 0xA000) as usize],
            
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            
            // Unusable memory; Returns 0xFF on DMG hardware
            0xFEA0..=0xFEFF => 0xFF,
            
            0xFF00..=0xFF7F => self.io[(address - 0xFF00) as usize],
            
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            
            0xFFFF => self.interrupt_enable,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {
                // TODO: Implement Memory Bank Controller (MBC) switching here
                // For now, standard ROMs are read-only, so we do nothing.
            }
            
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = value,
            
            0xA000..=0xBFFF => self.eram[(address - 0xA000) as usize] = value,
            
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            
            // 0xFEA0 - 0xFEFF: Unusable memory; Writes are ignored
            0xFEA0..=0xFEFF => {},
            
            0xFF00..=0xFF7F => {
                // Note: Writing to certain I/O registers triggers hardware behavior (like DMA transfers)
                // add specific hardware intercepts here later.
                self.io[(address - 0xFF00) as usize] = value;
            }
            
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            
            0xFFFF => self.interrupt_enable = value,
        }
    }
}