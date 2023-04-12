pub mod joypad;

use std::{rc::Rc, cell::RefCell};

use log::{warn, debug, error, trace};

use self::joypad::Joypad;

use super::{ppu::PPU, cartridge::Cartridge};

const CPU_MEMORY_SIZE: usize = 0x800;

const CPU_RAM_PAGE_START: usize = 0;
const CPU_RAM_PAGE_END: usize = 0x1FFF;
const CPU_RAM_MIRROR_MASK: usize = 0b0111_1111_1111;

const IO_PAGE_START: usize = 0x2008;
const IO_PAGE_END: usize = 0x3FFF;
const IO_MIRROR_MASK: usize = 0b10000000000111;

const PPU_CTRL_ADDRESS: usize = 0x2000;
const PPU_MASK_ADDRESS: usize = 0x2001;
const PPU_STATUS_ADDRESS: usize = 0x2002;
const OAM_ADDRESS: usize = 0x2003;
const OAM_DATA_ADDRESS: usize = 0x2004;
const PPU_SCROLL: usize = 0x2005;
const PPU_ADDRESS: usize = 0x2006;
const PPU_DATA_ADDRESS: usize = 0x2007;
const OAM_DMA_ADDRESS: usize = 0x4014;

// APU & I/O
const SQ1_VOL_ADDRESS: usize = 0x4000;
const SQ1_SWEEP_ADDRESS: usize = 0x4001;
const SQ1_LO_ADDRESS: usize = 0x4002;
const SQ1_HI_ADDRESS: usize = 0x4003;
const SQ2_VOL_ADDRESS: usize = 0x4004;
const SQ2_SWEEP_ADDRESS: usize = 0x4005;
const SQ2_LO_ADDRESS: usize = 0x4006;
const SQ2_HI_ADDRESS: usize = 0x4007;
const TRI_LINEAR_ADDRESS: usize = 0x4008;
const APU_UNUSED_REGISTER_1_ADDRESS: usize = 0x4009;
const TRI_LO_ADDRESS: usize = 0x400A;
const TRI_HI_ADDRESS: usize = 0x400B;
const NOISE_VOL_ADDRESS: usize = 0x400C;
const APU_UNUSED_REGISTER_2_ADDRESS: usize = 0x400D;
const NOISE_LO_ADDRESS: usize = 0x400E;
const NOISE_HI_ADDRESS: usize = 0x400F;
const DMC_FREQ_ADDRESS: usize = 0x4010;
const DMC_RAW_ADDRESS: usize = 0x4011;
const DMC_START_ADDRESS: usize = 0x4012;
const DMC_LEN_ADDRESS: usize = 0x4013;
// const OAM_DMA_ADDRESS: usize = 0x4014;
const SND_CHN_ADDRESS: usize = 0x4015;

const JOYPAD_1_IO_ADDRESS: usize = 0x4016;
const JOYPAD_2_IO_ADDRESS: usize = 0x4017;

const APU_IO_UNUSED_PAGE_START: usize = 0x4018;
const APU_IO_UNUSED_PAGE_END: usize = 0x401F;

const CARTRIDGE_PAGE_START: usize = 0x4020;
const CARTRIDGE_PAGE_END: usize = 0xFFFF;

const PROGRAM_ROM_PAGE_START: usize = 0x8000;
const PROGRAM_ROM_PAGE_END: usize = 0xFFFF;

pub struct Bus<'a> {
    pub cpu_memory: [u8; CPU_MEMORY_SIZE],
    pub cpu_cycles: u64,
    pub ppu: &'a mut PPU,
    pub cartridge: Rc<RefCell<Cartridge>>,

    pub joypads: [Joypad; 2],
}

impl<'a> Bus<'a> {
    pub fn new(ppu_device: &'a mut PPU, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        return Bus {
            cpu_memory: [0; CPU_MEMORY_SIZE],
            cpu_cycles: 0u64,
            ppu: ppu_device,
            cartridge: cartridge,
            joypads: [Joypad::new(); 2],
        };
    }

    pub fn load_rom(&mut self) {

    }

    pub fn poll_nmi_interrupt(&mut self) -> bool {
        return self.ppu.nmi_interrupt;
    }

    pub fn nmi_handled(&mut self) {
        self.ppu.nmi_interrupt = false;
    }

    pub fn read_memory_u8(&mut self, index: usize) -> u8 {
        match index {
            CPU_RAM_PAGE_START..=CPU_RAM_PAGE_END => {
                return self.cpu_memory[index & CPU_RAM_MIRROR_MASK];
            },
            PPU_CTRL_ADDRESS |
            PPU_MASK_ADDRESS |
            OAM_ADDRESS      |
            PPU_SCROLL       |
            PPU_ADDRESS      |
            OAM_DMA_ADDRESS => {
                warn!("Attempt to read temporary read-only area of PPU registers {:X}", index);
                
                return 0;
            },
            PPU_STATUS_ADDRESS => {
                return self.ppu.read_status_register();
            },
            OAM_DATA_ADDRESS => {
                return self.ppu.read_oam_data_register();
            },
            PPU_DATA_ADDRESS => {
                return self.ppu.read_data_register();
            },
            IO_PAGE_START..=IO_PAGE_END => {
                return self.read_memory_u8(index & IO_MIRROR_MASK);
            },
            SQ1_VOL_ADDRESS..=SND_CHN_ADDRESS => {
                warn!("Attempt to read unimplemented APU registers {:X}", index);

                return 0;
            },
            JOYPAD_1_IO_ADDRESS | JOYPAD_2_IO_ADDRESS => {
                warn!("Attempt to read joypad registers {:X}", index);
            
                let joypad_index = (index & 0x1) as usize;
            
                return self.joypads[joypad_index].read();
            },
            APU_IO_UNUSED_PAGE_START..=APU_IO_UNUSED_PAGE_END => {
                warn!("Attempt to read unused APU/IO memory {:X}", index);

                return 0;
            },
            PROGRAM_ROM_PAGE_START..=PROGRAM_ROM_PAGE_END => {
                debug!("Attempt to read program rom space {:X}", index);
                
                return self.cartridge.borrow().cpu_read_u8(index);
            },
            CARTRIDGE_PAGE_START..=usize::MAX => {
                warn!("Attempt to read unused cartridge (PRG ROM/RAM) space {:X}", index);

                return self.cartridge.borrow().cpu_read_u8(index);
            },
            _ => {
                panic!("Failed attempt to grab memory from {}", index);
            }
        }
    }

    pub fn read_memory_u16(&mut self, index: usize) -> u16 {
        return u16::from_le_bytes([
            self.read_memory_u8(index), 
            self.read_memory_u8(index + 1)
        ]);
    }

    pub fn write_memory_u8(&mut self, index: usize, value: u8) {
        match index {
            CPU_RAM_PAGE_START..=CPU_RAM_PAGE_END => {
                self.cpu_memory[index & CPU_RAM_MIRROR_MASK] = value;
            },
            PPU_CTRL_ADDRESS => {
                self.ppu.write_control_register(value);
            },
            PPU_MASK_ADDRESS => {
                self.ppu.write_mask_register(value);
            },
            OAM_ADDRESS => {
                self.ppu.write_oam_address_register(value);
            },
            OAM_DATA_ADDRESS => {
                self.ppu.write_oam_data_register(value);
            },
            PPU_SCROLL => {
                self.ppu.write_scroll_register(value);
            },
            PPU_ADDRESS => {
                self.ppu.write_address_register(value);
            },
            OAM_DMA_ADDRESS => {
                let mut cycles = 0;
                let start_address = (value as u16) << 8;
                let end_address = ((value as u16) << 8) | 0x00FF; 
                
                if self.cpu_cycles % 2 == 1 {
                    cycles += 1;
                    self.tick(1);
                }

                for address in start_address..=end_address {
                    let value = self.read_memory_u8(address as usize);
                    
                    self.tick(1);
                    cycles += 1;
                    
                    self.ppu.write_oam_dma_register_seq(value);
                    
                    self.tick(1);
                    cycles += 1;
                }
                trace!("OAM DMA write took {} cycles", cycles);
            },
            PPU_STATUS_ADDRESS => {
                warn!("Attempt to write to read-only ppu status register: {:X}; {:X}", index, value);
            },
            PPU_DATA_ADDRESS => {
                self.ppu.write_data_register(value);
            },
            IO_PAGE_START..=IO_PAGE_END => {
                // return self.read_memory_u8(index & IO_MIRROR_MASK);
            },
            SQ1_VOL_ADDRESS..=SND_CHN_ADDRESS => {
                warn!("Attempt to read unimplemented APU registers {:X}", index);

                // return 0;
            },
            JOYPAD_1_IO_ADDRESS | JOYPAD_2_IO_ADDRESS => {
                warn!("Attempt to read joypad registers {:X}", index);
                let joypad_index = (index & 0x1) as usize;
                
                self.joypads[joypad_index].write();
            },
            APU_IO_UNUSED_PAGE_START..=APU_IO_UNUSED_PAGE_END => {
                warn!("Attempt to write to unused APU/IO memory {:X}; value={:X}", index, value);

                // return 0;
            },
            CARTRIDGE_PAGE_START..=CARTRIDGE_PAGE_END => {
                warn!("Attempt to write to unused cartridge (PRG ROM/RAM) space {:X}; value={:X}", index, value);
                self.cartridge.borrow_mut().cpu_write_u8(index, value);
                // return 0
            },
            _ => {
                error!("Failed attempt to write memory to {:X}; value={:X}", index, value);
                panic!("Failed attempt to write memory to {:X}; value={:X}", index, value);
            }
        }
    }
    
    pub fn write_memory_u16(&mut self, index: usize, value: u16) {
        let [low, high] = u16::to_le_bytes(value);

        self.write_memory_u8(index, low);
        self.write_memory_u8(index + 1, high);
    }

    pub fn tick(&mut self, cycles: u64) {
        // if self.cpu_cycles == 134217730 || self.cpu_cycles == 134217730 {
        //     error!("Received zero cycles! {}; {}; {}", self.cpu_cycles, self.ppu.total_cycles, cycles);
        // }
        // if cycles.round() as usize == 0  || cycles as usize == 0{
        //     error!("Received zero cycles! {}; {}; {}", self.cpu_cycles, self.ppu.total_cycles, cycles)
        // }
        self.cpu_cycles += cycles as u64;
        // thrice the speed of cpu
        for _ in 0..(cycles * 3) {
            self.ppu.tick();
        }
        // self.ppu.tick( as usize);
        // two times slower than cpu
        // self.apu.tick(cycles);
    }

}