use super::Header;

pub enum MapperType {
    NROM = 0,
    UxRom = 2,
}

pub trait Mapper {
    fn map_cpu_read(&self, index: usize) -> usize;
    
    fn map_cpu_write(&mut self, index: usize, _value: u8) -> usize {
        return index;
    }

    fn map_ppu_read(&self, index: usize) -> usize;
    
    fn map_ppu_write(&mut self, index: usize, _value: u8) -> usize {
        return index;
    }
    
    fn has_ram(&self) -> bool {
        return false;
    }

    fn scanline(&mut self) {}
}

mod nrom_mapper {
    use crate::dendynes::cartridge::Header;

    pub const FIRST_PAGE_START: usize = 0x8000;
    pub const BANK_PAGE_SIZE: usize = 0x4000;
    pub const CHR_BANK_PAGE_SIZE: usize = 0x2000;

    pub struct NROMMapper {
        pub prg_banks_count: u8,
        pub chr_banks_count: u8,
    }
    
    impl NROMMapper {
        pub fn new(settings: Header) -> Self {
            return NROMMapper {
                prg_banks_count: settings.prg_banks_count,
                chr_banks_count: settings.chr_banks_count,
            };
        }
    }
}

mod uxrom_mapper {
    use crate::dendynes::cartridge::Header;

    pub const FIRST_ADDRESS_RANGE_START: usize = 0x8000;
    pub const FIRST_ADDRESS_RANGE_END: usize = 0xBFFF;

    pub const SECOND_ADDRESS_RANGE_START: usize = 0xC000;
    pub const SECOND_ADDRESS_RANGE_END: usize = 0xFFFF;

    pub const BANK_SIZE: usize = 0x4000;

    pub struct UxRomMapper {
        pub prg_banks_count: u8,
        pub chr_banks_count: u8,
        pub bank_select_register: u8,
        pub bank_select_count_register: u8,
    }

    impl UxRomMapper {
        pub fn new(settings: Header) -> Self {
            return UxRomMapper {
                prg_banks_count: settings.prg_banks_count,
                chr_banks_count: settings.chr_banks_count,
                bank_select_register: 0,
                bank_select_count_register: settings.prg_banks_count - 1,
            }; 
        }
    }
}   

impl Mapper for nrom_mapper::NROMMapper {
    fn map_cpu_read(&self, index: usize) -> usize {
        if self.prg_banks_count > 1 {
            let memory_span = nrom_mapper::BANK_PAGE_SIZE * self.prg_banks_count as usize;
            // debug!("Mapping: {:05X}; {:05X}; {:05X}; {:05X}", index, memory_span, memory_span - 1, index & (memory))
            return index & (memory_span - 1); 
        } else {
            return index & (nrom_mapper::BANK_PAGE_SIZE - 1);
        }
    }

    fn map_ppu_read(&self, index: usize) -> usize {
        if self.chr_banks_count > 1 {
            let memory_span = nrom_mapper::CHR_BANK_PAGE_SIZE * self.chr_banks_count as usize;
            
            return index & (memory_span & 1); 
        } else {
            return index & (nrom_mapper::CHR_BANK_PAGE_SIZE - 1);
        }
    }

    fn map_ppu_write(&mut self, index: usize, _value: u8) -> usize {
        if self.chr_banks_count > 1 {
            let memory_span = nrom_mapper::CHR_BANK_PAGE_SIZE * self.chr_banks_count as usize;
            
            return index & (memory_span & 1); 
        } else {
            return index & (nrom_mapper::CHR_BANK_PAGE_SIZE - 1);
        }
    }
}

impl Mapper for uxrom_mapper::UxRomMapper {
    fn map_cpu_read(&self, index: usize) -> usize {
        match index {
            uxrom_mapper::FIRST_ADDRESS_RANGE_START..=uxrom_mapper::FIRST_ADDRESS_RANGE_END => {
                let mapped = self.bank_select_register as usize * uxrom_mapper::BANK_SIZE + (index & (uxrom_mapper::BANK_SIZE - 1));
                // debug!!("Selecting low: {:05X}; {:05X}; {:05X}", self.bank_select_register, index, mapped);
                return mapped;
            },
            uxrom_mapper::SECOND_ADDRESS_RANGE_START..=uxrom_mapper::SECOND_ADDRESS_RANGE_END => {
                let mapped = self.bank_select_count_register as usize * uxrom_mapper::BANK_SIZE + (index & (uxrom_mapper::BANK_SIZE - 1));
                // debug!("Selecting high: {:05X}; {:05X}; {:05X}", self.bank_select_count_register, index, mapped);
                return mapped;
            },
            _ => {
                return index;
            }
        }
    }
    
    fn map_cpu_write(&mut self, index: usize, value: u8) -> usize {
        if index >= 0x8000 && index <= 0xFFFF {
            //debug!("Selecting different bank: {:05X}; {:02X}", index, value);
            self.bank_select_register = value & 0xF;
        }
    
        return index;
    }

    fn map_ppu_read(&self, index: usize) -> usize {
        return index;
    }
    
    fn map_ppu_write(&mut self, index: usize, value: u8) -> usize {
        return index;
    }

    fn has_ram(&self) -> bool {
        return true;
    }
}

pub fn new_mapper_by_type(mapper_type: MapperType, settings: Header) -> Box<dyn Mapper> {
    return match mapper_type {
        MapperType::NROM => Box::new(nrom_mapper::NROMMapper::new(settings)),
        MapperType::UxRom => Box::new(uxrom_mapper::UxRomMapper::new(settings)),
    };
}