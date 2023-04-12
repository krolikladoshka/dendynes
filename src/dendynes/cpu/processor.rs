use bitflags::bitflags;
use log::{warn, debug, info, trace, error};
use num_traits::FromPrimitive;

use crate::dendynes::{bus::Bus, memory::accessing_mode::MemoryAccessMode};

use super::opcode::{OpcodeType, OPCODES_MAP, Opcode};

const PROGRAM_POINTER_START: usize = 0xC000;
const STACK_PAGE_START: usize = 0x100;

const STACK_POINTER_START: usize = 0xFD;
const RESET_PROGRAM_POINTER_ADDRESS: usize = 0xFFFC;
const PAGE_CROSSING_CYCLES: u64 = 1;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct StatusFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const UNUSED            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

pub struct CPU<'a> {
    pub program_pointer: usize,

    pub stack_pointer: u8,

    pub cycles: u64,

    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    
    pub status: StatusFlags,
    
    pub bus: &'a mut Bus<'a>,
}

mod interrupts {

    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    pub enum InterruptType {
        BRK,
        IRQ,
        NMI,
    }

    pub const BRK_INTERRUPT_VECTOR_ADDRESS: u16 = 0xFFFE;
    pub const IRQ_INTERRUPT_VECTOR_ADDRESS: u16 = 0xFFFE;
    pub const NMI_INTERRUPT_VECTOR_ADDRESS: u16 = 0xFFFA;

    #[derive(Clone, Copy, Debug)]
    pub struct CpuInterrupt {
        pub interrupt_type: InterruptType,
        pub vector_addr: u16,
        pub cycles: u64,
    }

    pub const BRK: CpuInterrupt = CpuInterrupt {
        interrupt_type: InterruptType::BRK,
        vector_addr: BRK_INTERRUPT_VECTOR_ADDRESS,
        cycles: 1,
    };

    pub const IRQ: CpuInterrupt = CpuInterrupt {
        interrupt_type: InterruptType::IRQ,
        vector_addr: IRQ_INTERRUPT_VECTOR_ADDRESS,
        cycles: (7 / 3) + 1,
    };

    pub const NMI: CpuInterrupt = CpuInterrupt {
        interrupt_type: InterruptType::NMI,
        vector_addr: NMI_INTERRUPT_VECTOR_ADDRESS,
        cycles: (8 / 3) + 1,
    };
}

impl<'a> CPU<'a> {
    pub fn new(bus: &'a mut Bus<'a>) -> Self {
        let mut cpu = CPU {
            program_pointer: PROGRAM_POINTER_START,
            stack_pointer: STACK_POINTER_START as u8,
            cycles: 0,
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: StatusFlags::from_bits(0x24).unwrap(),
            bus: bus,
        };
        cpu.reset();
        // cpu.bus.tick(7f32);

        return cpu;
    }

    pub fn reset(&mut self) {
        warn!("Resetting the processor");
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = STACK_POINTER_START as u8;

        debug!("Reading program start address from {:X}", RESET_PROGRAM_POINTER_ADDRESS);
        self.program_pointer = self.bus.read_memory_u16(RESET_PROGRAM_POINTER_ADDRESS) as usize;

        // self.program_pointer = PROGRAM_POINTER_START;
        self.status = StatusFlags::empty();
        self.set_unused_status();
        // self.set_interrupt_disable_status();
        
        trace!("CPU dump after reset: PC-{:X} | A:{:X} X:{:X} Y:{:X} P:{:X} SP:{:X};  Status: {:?}",
            self.program_pointer, self.register_a, self.register_x, self.register_y,
            self.status.bits(), self.stack_pointer, self.status
        );
        
        self.bus.tick(8);
    }

    pub fn get_opcode_data_address(&mut self, index: usize, access_mode: MemoryAccessMode, read: bool) -> u16 {
        let index = index as u16;
    
        match access_mode {
            MemoryAccessMode::Implied => {
                return 0;
            },
            MemoryAccessMode::Immediate => {
                return index;
            },
            MemoryAccessMode::Accumulator => {
                return self.register_a as u16;
            },
            MemoryAccessMode::ZeroPage => {
                let address = self.bus.read_memory_u8(index as usize);
                return (address & 0xFF) as u16;
            },
            MemoryAccessMode::ZeroPageX => {
                let address = self.bus.read_memory_u8(index as usize).wrapping_add(self.register_x);
                return (address & 0xFF) as u16;
            },
            MemoryAccessMode::ZeroPageY => {
                let address = self.bus.read_memory_u8(index as usize).wrapping_add(self.register_y);
                return (address & 0xFF) as u16;
            },
            MemoryAccessMode::Relative(crossing_page) => {
                let mut address = self.bus.read_memory_u8(index as usize) as u16;

                if (address & 0x80) == 1 {
                    address |= 0xFF00;
                }

                return address
            },
            MemoryAccessMode::Absolute => {
                let address = self.bus.read_memory_u16(index as usize);
                return address;
            },
            MemoryAccessMode::AbsoluteX(crossing_page) => {
                let address = self.bus.read_memory_u16(index as usize);
                let added_address = address.wrapping_add(self.register_x as u16);

                if read && crossing_page {
                    self.cross_page(address, added_address);
                }

                return added_address;
            },
            MemoryAccessMode::AbsoluteY(crossing_page) => {
                let address = self.bus.read_memory_u16(index as usize);
                let added_address = address.wrapping_add(self.register_y as u16);

                if read && crossing_page {
                    self.cross_page(address, added_address);
                }

                return added_address;
            },
            MemoryAccessMode::Indirect => {
                let low = self.bus.read_memory_u8(index as usize);
                let high = self.bus.read_memory_u8((index + 1) as usize);
                let pointer = u16::from_le_bytes([low, high]);

                if low == 0xFF {
                    let address = u16::from_le_bytes([
                        self.bus.read_memory_u8(pointer as usize),
                        self.bus.read_memory_u8((pointer & 0xFF00) as usize)
                    ]);
                    warn!(
                        "Bugged indirect; pointer: {:04X}; address: {:04X}",
                        pointer, address
                    );

                    return address;
                } else {
                    return self.bus.read_memory_u16(pointer as usize);
                }
            },
            MemoryAccessMode::IndirectX => {
                let pointer = self.bus.read_memory_u8(index as usize).wrapping_add(self.register_x);
                
                return u16::from_le_bytes([
                    self.bus.read_memory_u8(pointer as usize),
                    self.bus.read_memory_u8(pointer.wrapping_add(1) as usize),
                ]);
            },
            MemoryAccessMode::IndirectY(crossing_page) => {
                let pointer = self.bus.read_memory_u8(index as usize);
                let address = u16::from_le_bytes([
                    self.bus.read_memory_u8(pointer as usize),
                    self.bus.read_memory_u8(pointer.wrapping_add(1) as usize),
                ]);
                
                let added_address = address.wrapping_add(self.register_y as u16);
                
                if read && crossing_page {
                    self.cross_page(address, added_address);
                }

                return added_address;
            },
        }
    }

    pub fn cross_page(&mut self, base: u16, added: u16) {
        if MemoryAccessMode::page_crossed(base as usize, added as usize) {
            self.bus.tick(PAGE_CROSSING_CYCLES);
        }
    }

    pub fn read_for_trace(&mut self, index: usize, access_mode: MemoryAccessMode) -> u8 {
        match access_mode {
            MemoryAccessMode::Accumulator => {
                return self.register_a;
            },
            MemoryAccessMode::Immediate => {
                return self.bus.read_memory_u8(self.program_pointer);
            },
            _ => {
                let address = self.get_opcode_data_address(index, access_mode, false);

                return self.bus.read_memory_u8(address as usize);
            }
        }
    }

    pub fn read_opcode_data(&mut self, index: usize, access_mode: MemoryAccessMode) -> u8 {
        match access_mode {
            MemoryAccessMode::Accumulator => {
                return self.register_a;
            },
            MemoryAccessMode::Immediate => {
                return self.bus.read_memory_u8(self.program_pointer);
            },
            _ => {
                let address = self.get_opcode_data_address(index, access_mode, true);

                return self.bus.read_memory_u8(address as usize);
            }
        }
    }

    pub fn write_opcode_result(&mut self, index: usize, value: u8, access_mode: MemoryAccessMode) {
        match access_mode {
            MemoryAccessMode::Implied => {
                error!("Somehow trying to write memory to Implied location");
            },
            MemoryAccessMode::Accumulator => {
                self.set_register_a(value);
            },
            MemoryAccessMode::Immediate => {
                self.bus.write_memory_u8(self.program_pointer, value);
            },
            _ => {
                let address = self.get_opcode_data_address(index, access_mode, false);

                self.bus.write_memory_u8(address as usize, value);
            }
        }
    }

    pub fn run(&mut self) {
        // self.reset();
    
        loop {
            self.cpu_step();
        }
    }

    // pub fn step_for_cycles(&mut self, cycles: usize) {
    //     let cycles = cycles as u128;
    //     let cycles_since_start = self.bus.cpu_cycles;
        
    //     let mut elapsed_cycles = 
    // }
    pub fn cpu_step(&mut self) -> u64 {
        if self.bus.poll_nmi_interrupt() {
            self.interrupt(interrupts::NMI);
            self.bus.nmi_handled();
        }
        
        return self.execute_opcode();
    }

    fn trace_state(&mut self) {
        let opcode_raw = self.bus.read_memory_u8(self.program_pointer);
        let opcode = match OpcodeType::from_u8(opcode_raw) {
            Some(op) => op,
            None => panic!("Found unknown opcode {}", opcode_raw),
        };
        
        let opcode_metadata = match OPCODES_MAP.get(&opcode) {
            Some(op) => op,
            None => {
                trace!("Could not find opcode {:?} ({:02X}) in opcodes map", opcode, opcode_raw);
                panic!("Could not find opcode {:?} ({:02X}) in opcodes map", opcode, opcode_raw);
            }
        };

        match opcode_metadata.memory_mode {
            MemoryAccessMode::Absolute | MemoryAccessMode::AbsoluteX(_) | MemoryAccessMode::AbsoluteY(_) |
            MemoryAccessMode::Indirect => {
                let [low, high] = u16::to_le_bytes(self.bus.read_memory_u16(self.program_pointer + 1));

                trace!(
                    "{:04X}  {:02X} {: <02X} {: <02X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                    self.program_pointer, opcode_raw, low, high,
                    self.register_a, self.register_x, self.register_y,
                    self.status.bits(), self.stack_pointer, self.bus.cpu_cycles,
                );
                // info!(
                //     "{:04X}  {:02X} {: <02X} {: <02X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                //     self.program_pointer, opcode_raw, low, high,
                //     self.register_a, self.register_x, self.register_y,
                //     self.status.bits(), self.stack_pointer, self.bus.cpu_cycles,
                // );
            },
            MemoryAccessMode::Immediate | MemoryAccessMode::ZeroPage | MemoryAccessMode::ZeroPageX |
            MemoryAccessMode::ZeroPageY | MemoryAccessMode::Relative(_) | MemoryAccessMode::IndirectX |
            MemoryAccessMode::IndirectY(_) => {
                let arg =self.bus.read_memory_u8(self.program_pointer + 1);

                trace!(
                    "{:04X}  {:02X} {: <02X}     A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                    self.program_pointer, opcode_raw, arg,
                    self.register_a, self.register_x, self.register_y,
                    self.status.bits(), self.stack_pointer, self.bus.cpu_cycles
                );
                // info!(
                //     "{:04X}  {:02X} {: <02X}     A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                //     self.program_pointer, opcode_raw, arg,
                //     self.register_a, self.register_x, self.register_y,
                //     self.status.bits(), self.stack_pointer, self.bus.cpu_cycles
                // );
            },
            MemoryAccessMode::Implied | MemoryAccessMode::Accumulator => {
                trace!(
                    "{:04X}  {:02X}        A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                    self.program_pointer, opcode_raw,
                    self.register_a, self.register_x, self.register_y,
                    self.status.bits(), self.stack_pointer, self.bus.cpu_cycles
                );
                // info!(
                //     "{:04X}  {:02X}        A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                //     self.program_pointer, opcode_raw,
                //     self.register_a, self.register_x, self.register_y,
                //     self.status.bits(), self.stack_pointer, self.bus.cpu_cycles
                // );
            }
        };
    }

    pub fn execute_opcode(&mut self) -> u64 {
        debug!("!! Execute opcode start !!");
        // trace!("!!!Program Pointer: {:04X}", self.program_pointer);

        let cycles = self.bus.cpu_cycles;
        
        self.trace_state();
        
        let opcode = self.bus.read_memory_u8(self.program_pointer);
        debug!("Read opcode {}", opcode);
        let opcode = match OpcodeType::from_u8(opcode) {
            Some(op) => op,
            None => panic!("Found unknown opcode {}", opcode),
        };

        debug!(
            "CPU dump: PC-{:04X} {:?} | A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X};  Status: {:?}",
            self.program_pointer, opcode, self.register_a, self.register_x, self.register_y,
            self.status.bits(), self.stack_pointer, self.status,
        );
        debug!("Executing converted opcode {:?}; pc {:X}", opcode, self.program_pointer);
        self.program_pointer += 1;
        debug!("Advanced PC {:X}", self.program_pointer);
        
        let opcode_metadata = OPCODES_MAP.get(&opcode).unwrap();

        debug!("Metadata {:?}", opcode_metadata);

        let program_pointer = self.program_pointer;

        match opcode {
            OpcodeType::Brk => {
                self.brk();
            },
            OpcodeType::AdcI | OpcodeType::AdcZp | OpcodeType::AdcZpx |
            OpcodeType::AdcA | OpcodeType::AdcAx | OpcodeType::AdcAy |
            OpcodeType::AdcIx | OpcodeType::AdcIy => {
                self.adc(opcode_metadata.memory_mode);
            },
            OpcodeType::AndI | OpcodeType::AndZp | OpcodeType::AndZpx |
            OpcodeType::AndA | OpcodeType::AndAx | OpcodeType::AndAy |
            OpcodeType::AndIx | OpcodeType::AndIy => {
                self.and(opcode_metadata.memory_mode);
            },
            OpcodeType::AslAcc | OpcodeType::AslZp | OpcodeType::AslZpx |
            OpcodeType::AslA | OpcodeType::AslAx => {
                self.asl(opcode_metadata.memory_mode);
            },
            OpcodeType::LsrAcc | OpcodeType::LsrZp | OpcodeType::LsrZpx |
            OpcodeType::LsrA | OpcodeType::LsrAx => {
                self.lsr(opcode_metadata.memory_mode);
            },
            OpcodeType::Bcc => {
                self.bcc(opcode_metadata.memory_mode);
            },
            OpcodeType::Bcs => {
                self.bcs(opcode_metadata.memory_mode);
            },
            OpcodeType::Beq => {
                self.beq(opcode_metadata.memory_mode);
            },
            OpcodeType::BitZp | OpcodeType::BitA => {
                self.bit(opcode_metadata.memory_mode);
            },
            OpcodeType::Bmi => {
                self.bmi(opcode_metadata.memory_mode);
            },
            OpcodeType::Bne => {
                self.bne(opcode_metadata.memory_mode);
            },
            OpcodeType::Bpl => {
                self.bpl(opcode_metadata.memory_mode);
            },
            OpcodeType::Bvc => {
                self.bvc(opcode_metadata.memory_mode);
            },
            OpcodeType::Bvs => {
                self.bvs(opcode_metadata.memory_mode);
            },
            OpcodeType::Clc => {
                self.clc(opcode_metadata.memory_mode);
            },
            OpcodeType::Cld => {
                self.cld(opcode_metadata.memory_mode);
            },
            OpcodeType::Cli => {
                self.cli(opcode_metadata.memory_mode);
            },
            OpcodeType::Clv => {
                self.clv(opcode_metadata.memory_mode);
            },
            OpcodeType::CmpI | OpcodeType::CmpZp | OpcodeType::CmpZpx |
            OpcodeType::CmpA | OpcodeType::CmpAx | OpcodeType::CmpAy |
            OpcodeType::CmpIx | OpcodeType::CmpIy => {
                self.cmp(opcode_metadata.memory_mode);
            },
            OpcodeType::CpxI | OpcodeType::CpxZp | OpcodeType::CpxA => {
                self.cpx(opcode_metadata.memory_mode);
            },
            OpcodeType::CpyI | OpcodeType::CpyZp | OpcodeType::CpyA => {
                self.cpy(opcode_metadata.memory_mode);
            },
            OpcodeType::DecZp | OpcodeType::DecZpx | OpcodeType::DecA | 
            OpcodeType::DecAx => {
                self.dec(opcode_metadata.memory_mode);
            },
            OpcodeType::Dex => {
                self.dex(opcode_metadata.memory_mode);
            },
            OpcodeType::Dey => {
                self.dey(opcode_metadata.memory_mode);
            },
            OpcodeType::IncZp | OpcodeType::IncZpx | OpcodeType::IncA |
            OpcodeType::IncAx => {
                self.inc(opcode_metadata.memory_mode);
            },
            OpcodeType::Inx => {
                self.inx(opcode_metadata.memory_mode);
            },
            OpcodeType::Iny => {
                self.iny(opcode_metadata.memory_mode);
            },
            OpcodeType::EorI | OpcodeType::EorZp | OpcodeType::EorZpx |
            OpcodeType::EorA | OpcodeType::EorAx | OpcodeType::EorAy |
            OpcodeType::EorIx | OpcodeType::EorIy => {
                self.eor(opcode_metadata.memory_mode);
            },
            OpcodeType::JmpA | OpcodeType::JmpInd => {
                self.jmp(opcode_metadata.memory_mode);
            },
            OpcodeType::Jsr => {
                self.jsr(opcode_metadata);
            },
            OpcodeType::LdaI | OpcodeType::LdaZp | OpcodeType::LdaZpx |
            OpcodeType::LdaA | OpcodeType::LdaAx | OpcodeType::LdaAy | 
            OpcodeType::LdaIx | OpcodeType::LdaIy => {
                self.lda(opcode_metadata.memory_mode);
            },
            OpcodeType::LdxI | OpcodeType::LdxZp | OpcodeType::LdxZpy |
            OpcodeType::LdxA | OpcodeType::LdxAy => {
                self.ldx(opcode_metadata.memory_mode);
            },
            OpcodeType::LdyI | OpcodeType::LdyZp | OpcodeType::LdyZpx |
            OpcodeType::LdyA | OpcodeType::LdyAx => {
                self.ldy(opcode_metadata.memory_mode);
            },
            OpcodeType::Nop | OpcodeType::Nop1 | OpcodeType::Nop2 |
            OpcodeType::Nop3 | OpcodeType::Nop4 | OpcodeType::Nop5 |
            OpcodeType::Nop6 | OpcodeType::NopHlt => {
                self.nop(opcode_metadata.memory_mode);
            },
            OpcodeType::OraI | OpcodeType::OraZp | OpcodeType::OraZpx |
            OpcodeType::OraA | OpcodeType::OraAx | OpcodeType::OraAy |
            OpcodeType::OraIx | OpcodeType::OraIy => {
                self.ora(opcode_metadata.memory_mode);
            },
            OpcodeType::Pha => {
                self.pha(opcode_metadata.memory_mode);
            },
            OpcodeType::Php => {
                self.php(opcode_metadata.memory_mode);
            },
            OpcodeType::Pla => {
                self.pla(opcode_metadata.memory_mode);
            },
            OpcodeType::Plp => {
                self.plp(opcode_metadata.memory_mode);
            },
            OpcodeType::RolAcc | OpcodeType::RolZp | OpcodeType::RolZpx |
            OpcodeType::RolA | OpcodeType::RolAx => {
                self.rol(opcode_metadata.memory_mode);
            },
            OpcodeType::RorAcc | OpcodeType::RorZp | OpcodeType::RorZpx |
            OpcodeType::RorA | OpcodeType::RorAx => {
                self.ror(opcode_metadata.memory_mode);
            },
            OpcodeType::Rti => {
                self.rti(opcode_metadata.memory_mode);
            },
            OpcodeType::Rts => {
                self.rts(opcode_metadata.memory_mode);
            },
            OpcodeType::SbcI | OpcodeType::SbcZp | OpcodeType::SbcZpx |
            OpcodeType::SbcA | OpcodeType::SbcAx | OpcodeType::SbcAy |
            OpcodeType::SbcIx | OpcodeType::SbcIy => {
                self.sbc(opcode_metadata.memory_mode);
            },
            OpcodeType::Sec => {
                self.sec(opcode_metadata.memory_mode);
            },
            OpcodeType::Sed => {
                self.sed(opcode_metadata.memory_mode);
            },
            OpcodeType::Sei => {
                self.sei(opcode_metadata.memory_mode);
            },
            OpcodeType::StaZp | OpcodeType::StaZpx | OpcodeType::StaA |
            OpcodeType::StaAx | OpcodeType::StaAy | OpcodeType::StaIx |
            OpcodeType::StaIy => {
                self.sta(opcode_metadata.memory_mode);
            },
            OpcodeType::StxZp | OpcodeType::StxZpy | OpcodeType::StxA => {
                self.stx(opcode_metadata.memory_mode);
            },
            OpcodeType::StyZp | OpcodeType::StyZpx | OpcodeType::StyA => {
                self.sty(opcode_metadata.memory_mode);
            },
            OpcodeType::Tax => {
                self.tax(opcode_metadata.memory_mode);
            },
            OpcodeType::Tay => {
                self.tay(opcode_metadata.memory_mode);
            },
            OpcodeType::Tsx => {
                self.tsx(opcode_metadata.memory_mode);
            },
            OpcodeType::Txa => {
                self.txa(opcode_metadata.memory_mode);
            },
            OpcodeType::Txs => {
                self.txs(opcode_metadata.memory_mode);
            },
            OpcodeType::Tya => {
                self.tya(opcode_metadata.memory_mode);
            },
            OpcodeType::AsoZp | OpcodeType::AsoZpx | OpcodeType::AsoA |
            OpcodeType::AsoAx | OpcodeType::AsoAy | OpcodeType::AsoIx |
            OpcodeType::AsoIy => {
                self.aso(opcode_metadata.memory_mode);
            },
            OpcodeType::RlaZp | OpcodeType::RlaZpx | OpcodeType::RlaA |
            OpcodeType::RlaAx | OpcodeType::RlaAy | OpcodeType::RlaIx | 
            OpcodeType::RlaIy => {
                self.rla(opcode_metadata.memory_mode);
            },
            OpcodeType::LseZp | OpcodeType::LseZpx | OpcodeType::LseA |
            OpcodeType::LseAx | OpcodeType::LseAy | OpcodeType::LseIx |
            OpcodeType::LseIy => {
                self.lse(opcode_metadata.memory_mode);
            },
            OpcodeType::RraZp | OpcodeType::RraZpx | OpcodeType::RraA |
            OpcodeType::RraAx | OpcodeType::RraAy | OpcodeType::RraIx |
            OpcodeType::RraIy => {
                self.rra(opcode_metadata.memory_mode);
            },
            OpcodeType::AxsZp | OpcodeType::AxsZpy | OpcodeType::AxsA |
            OpcodeType::AxsIx => {
                self.axs(opcode_metadata.memory_mode);
            },
            OpcodeType::LaxZp | OpcodeType::LaxZpy | OpcodeType::LaxA |
            OpcodeType::LaxAy | OpcodeType::LaxIx | OpcodeType::LaxIy => {
                self.lax(opcode_metadata.memory_mode);
            },
            OpcodeType::Skb | OpcodeType::Skb1 | OpcodeType::Skb2 |
            OpcodeType::Skb3 | OpcodeType::Skb4 | OpcodeType::Skb5 |
            OpcodeType::Skb6 | OpcodeType::Skb7 | OpcodeType::Skb8 |
            OpcodeType::Skb9 | OpcodeType::Skb10 | OpcodeType::Skb11 |
            OpcodeType::Skb12 | OpcodeType::Skb13 | OpcodeType::Skb14 |
            OpcodeType::Skb15 | OpcodeType::Skb16 | OpcodeType::Skb17 |
            OpcodeType::Skb18 | OpcodeType::Skb19 | OpcodeType::Skb20 => {
                self.skb(opcode_metadata.memory_mode)
            },
            OpcodeType::Sbc1I => {
                self.sbc(opcode_metadata.memory_mode);
            },
            OpcodeType::DcmZp | OpcodeType::DcmZpx | OpcodeType::DcmA |
            OpcodeType::DcmAx | OpcodeType::DcmAy | OpcodeType::DcmIx |
            OpcodeType::DcmIy => {
                self.dcm(opcode_metadata.memory_mode);
            },
            OpcodeType::InsZp | OpcodeType::InsZpx | OpcodeType::InsA |
            OpcodeType::InsAx | OpcodeType::InsAy | OpcodeType::InsIx |
            OpcodeType::InsIy => {
                self.ins(opcode_metadata.memory_mode);
            },
            OpcodeType::AxaAy | OpcodeType::AxaIy => {
                self.axa(opcode_metadata.memory_mode);
            },
            OpcodeType::Say => {
                self.say(opcode_metadata.memory_mode);
            },
            _ => panic!("Unknown opcode {:?}", opcode),
        }

        self.cycles += opcode_metadata.cycles as u64;
        self.bus.tick(opcode_metadata.cycles as u64);
        
        if program_pointer != self.program_pointer {
            // trace!("There was a jump from {:X} to {:X}", program_pointer, self.program_pointer);
            debug!("There was a jump from {:X} to {:X}", program_pointer, self.program_pointer);
        } else {
            // trace!("Advancing PC from {:X} to {:X}", self.program_pointer, self.program_pointer + (opcode_metadata.length - 1) as usize);
            debug!("Advancing PC from {:X} to {:X}", self.program_pointer, self.program_pointer + (opcode_metadata.length - 1) as usize);
            self.program_pointer += (opcode_metadata.length - 1) as usize;

        }

        return self.bus.cpu_cycles - cycles;
        // return 0.0f32;
    }

    pub fn set_carry_status(&mut self) {
        self.status.insert(StatusFlags::CARRY);
    }

    pub fn clear_carry_status(&mut self) {
        self.status.remove(StatusFlags::CARRY);
    }

    pub fn set_zero_status(&mut self) {
        self.status.insert(StatusFlags::ZERO);
    }

    pub fn clear_zero_status(&mut self) {
        self.status.remove(StatusFlags::ZERO);
    }

    pub fn set_interrupt_disable_status(&mut self) {
        self.status.insert(StatusFlags::INTERRUPT_DISABLE);
    }

    pub fn clear_interrupt_disable_status(&mut self) {
        self.status.remove(StatusFlags::INTERRUPT_DISABLE);
    }
    
    pub fn set_decimal_mode_status(&mut self) {
        self.status.insert(StatusFlags::DECIMAL_MODE);
    }

    pub fn clear_decimal_mode_status(&mut self) {
        self.status.remove(StatusFlags::DECIMAL_MODE);
    }

    pub fn set_break_status(&mut self) {
        self.status.insert(StatusFlags::BREAK);
    }

    pub fn clear_break_status(&mut self) {
        self.status.remove(StatusFlags::BREAK);
    }

    pub fn set_unused_status(&mut self) {
        self.status.insert(StatusFlags::UNUSED);
    }

    pub fn clear_unused_status(&mut self) {
        self.status.remove(StatusFlags::UNUSED);
    }

    pub fn set_overflow_status(&mut self) {
        self.status.insert(StatusFlags::OVERFLOW);
    }

    pub fn clear_overflow_status(&mut self) {
        self.status.remove(StatusFlags::OVERFLOW);
    }

    pub fn set_negative_status(&mut self) {
        self.status.insert(StatusFlags::NEGATIVE);
    }

    pub fn clear_negative_status(&mut self) {
        self.status.remove(StatusFlags::NEGATIVE);
    }

    pub fn update_negative_status(&mut self, result: u8) {
        if (result >> 7) == 1 {
            self.set_negative_status();
        } else {
            self.clear_negative_status();
        }
    }

    pub fn update_zero_status(&mut self, result: u8) {
        if result == 0 {
            self.set_zero_status();
        } else {
            self.clear_zero_status();
        }
    }

    pub fn update_status(&mut self, result: u8) {
        self.update_negative_status(result);
        self.update_zero_status(result);        
    }

    pub fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_status(value);
    }

    pub fn or_register_a(&mut self, value: u8) {
        let result = self.register_a | value;

        self.set_register_a(result);
    }

    pub fn and_register_a(&mut self, value: u8) {
        let result = self.register_a & value;
        
        self.set_register_a(result);
    }

    pub fn xor_register_a(&mut self, value: u8) {
        let result = self.register_a ^ value;

        self.set_register_a(result);
    }

    pub fn set_register_x(&mut self, value: u8) {
        self.register_x = value;
        self.update_status(value);
    }
    
    pub fn set_register_y(&mut self, value: u8) {
        self.register_y = value;
        self.update_status(value);
    }
    
    pub fn push_stack_u8(&mut self, value: u8) {
        self.bus.write_memory_u8(STACK_PAGE_START + self.stack_pointer as usize, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn push_stack_u16(&mut self, value: u16) {
        let [low, high] = u16::to_le_bytes(value);

        self.push_stack_u8(high);
        self.push_stack_u8(low);
    }
    
    pub fn pop_stack_u8(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);

        return self.bus.read_memory_u8(STACK_PAGE_START + self.stack_pointer as usize);
    }

    pub fn pop_stack_u16(&mut self) -> u16 {
        let low = self.pop_stack_u8();
        let high = self.pop_stack_u8();

        return u16::from_le_bytes([low, high]);
    }

    pub fn interrupt(&mut self, interrupt: interrupts::CpuInterrupt) {
        debug!("Interrupt {:?}; pushing PC {:04X}", interrupt, self.program_pointer);
        self.push_stack_u16(self.program_pointer as u16);
                
        if interrupt.interrupt_type == interrupts::InterruptType::BRK {
            self.set_break_status();
        } else {
            self.clear_break_status();
        }
        self.set_interrupt_disable_status();
        self.set_unused_status();
        self.push_stack_u8(self.status.bits());
        
        self.bus.tick(interrupt.cycles);

        self.program_pointer = self.bus.read_memory_u16(interrupt.vector_addr as usize) as usize;

        debug!("End Interrupt {:?}; new PC {:04X}", interrupt, self.program_pointer);
    }

    fn brk(&mut self) {
        self.program_pointer += 1;
        
        if self.status.contains(StatusFlags::INTERRUPT_DISABLE) {
            return;
        }

        self.interrupt(interrupts::BRK);
    }

    fn add_to_register_a(&mut self, value: u8) {
        let overflowing_bit = if self.status.contains(StatusFlags::CARRY) {
            1u16
        } else {
            0u16
        };
        let result = self.register_a as u16 + value as u16 + overflowing_bit;

        // let result = (value as u16) + overflowing_bit + self.register_a as u16;
        
        if result > u8::MAX as u16 {
            self.set_carry_status();
        } else {
            self.clear_carry_status()
        }
        
        if (result & u8::MAX as u16) == 1 {
            self.set_zero_status();
        } else {
            self.clear_zero_status();
        }

        if result & 0x80 > 0 {
            self.set_negative_status();
        } else {
            self.clear_negative_status();
        }
        
        let result = result as u8;
        if (value ^ result) & (result ^ self.register_a) & 0x80 > 0 {
            self.set_overflow_status();
        } else {
            self.clear_overflow_status();
        }

        self.set_register_a(result);
    }

    fn sub_from_register_a(&mut self, value: u8) {
        let value = (value as i8).wrapping_neg().wrapping_sub(1) as u8;

        self.add_to_register_a(value);
    }
    
    /* +unofficial */
    fn axa(&mut self, memory_mode: MemoryAccessMode) {
        let address = self.get_opcode_data_address(self.program_pointer, memory_mode, true);
        let [_, high] = u16::to_le_bytes(address);
        let result = (self.register_a & self.register_x & high).wrapping_add(1);

        self.write_opcode_result(self.program_pointer, result, memory_mode);
    }

    fn say(&mut self, memory_mode: MemoryAccessMode) {
        let address = self.get_opcode_data_address(self.program_pointer, memory_mode, true);
        let [_, high] = u16::to_le_bytes(address);
        let result = self.register_y & (high + 1);

        self.write_opcode_result(self.program_pointer, result, memory_mode);
    }

    fn aso(&mut self, memory_mode: MemoryAccessMode) {
        let result = self.asl(memory_mode);
        self.or_register_a(result);
    }

    fn rla(&mut self, memory_mode: MemoryAccessMode) {
        let result = self.rol(memory_mode);
        self.and_register_a(result);
    }

    fn lse(&mut self, memory_mode: MemoryAccessMode) {
        let result = self.lsr(memory_mode);
        self.xor_register_a(result);
    }

    fn rra(&mut self, memory_mode: MemoryAccessMode) {
        let result = self.ror(memory_mode);
        self.add_to_register_a(result);
    }

    fn axs(&mut self, memory_mode: MemoryAccessMode) {
        let result = self.register_a & self.register_x;
        self.write_opcode_result(self.program_pointer, result, memory_mode);
    }

    fn lax(&mut self, memory_mode: MemoryAccessMode) {
        self.lda(memory_mode);
        self.set_register_x(self.register_a);
    }

    fn skb(&mut self, memory_mode: MemoryAccessMode) {
        // tick & advance
        self.read_opcode_data(self.program_pointer, memory_mode);
    }
    /* -unofficial */
    
    /* +loads */
    fn lda(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);

        self.set_register_a(opcode_data);
    }
    
    fn ldx(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);

        self.set_register_x(opcode_data);
    }

    fn ldy(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);

        self.set_register_y(opcode_data);
    }
    /* -loads */
    
    /* +stores */
    fn sta(&mut self, memory_mode: MemoryAccessMode) {
        self.write_opcode_result(
            self.program_pointer, self.register_a, memory_mode
        );
    }

    fn stx(&mut self, memory_mode: MemoryAccessMode) {
        self.write_opcode_result(
            self.program_pointer, self.register_x, memory_mode
        );
    }

    fn sty(&mut self, memory_mode: MemoryAccessMode) {
        self.write_opcode_result(
            self.program_pointer, self.register_y, memory_mode
        );
    }

    fn tax(&mut self, memory_mode: MemoryAccessMode) {
        self.set_register_x(self.register_a);
    }
    
    fn tay(&mut self, memory_mode: MemoryAccessMode) {
        self.set_register_y(self.register_a);
    }

    fn tsx(&mut self, memory_mode: MemoryAccessMode) {
        self.set_register_x(self.stack_pointer);
    }
    
    fn txa(&mut self, memory_mode: MemoryAccessMode) {
        self.set_register_a(self.register_x);
    }

    fn txs(&mut self, memory_mode: MemoryAccessMode) {
        self.stack_pointer = self.register_x;
    }

    fn tya(&mut self, memory_mode: MemoryAccessMode) {
        self.set_register_a(self.register_y);
    }
    /* -stores */
    
    /* +arith */
    fn adc(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);

        self.add_to_register_a(opcode_data);
    }

    fn dcm (&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = opcode_data.wrapping_sub(1);

        self.write_opcode_result(self.program_pointer, result, memory_mode);

        
        if self.register_a >= result {
            self.set_carry_status();
        } else {
            self.clear_carry_status();
        }
        self.update_status(self.register_a.wrapping_sub(result));
    }

    fn ins(&mut self, memory_mode: MemoryAccessMode) {
        let result = self.inc(memory_mode);
        self.sub_from_register_a(result);
    }

    fn sbc(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        self.sub_from_register_a(opcode_data);
    }

    fn dec(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = opcode_data.wrapping_sub(1);

        self.update_status(result);

        self.write_opcode_result(self.program_pointer, result, memory_mode);
    }

    fn dex(&mut self, memory_mode: MemoryAccessMode) {
        self.register_x = self.register_x.wrapping_sub(1);

        self.update_status(self.register_x);
    }

    fn dey(&mut self, memory_mode: MemoryAccessMode) {
        self.register_y = self.register_y.wrapping_sub(1);

        self.update_status(self.register_y);
    }

    fn inc(&mut self, memory_mode: MemoryAccessMode) -> u8 {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = opcode_data.wrapping_add(1);

        self.update_status(result);

        self.write_opcode_result(self.program_pointer, result, memory_mode);

        return result;
    }

    fn inx(&mut self, memory_mode: MemoryAccessMode) {
        self.register_x = self.register_x.wrapping_add(1);

        self.update_status(self.register_x);
    }

    fn iny(&mut self, memory_mode: MemoryAccessMode) {
        self.register_y = self.register_y.wrapping_add(1);

        self.update_status(self.register_y);
    }
    /* -arith */

    /* +bits */
    fn and(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = self.register_a & opcode_data;

        self.set_register_a(result);
    }

    fn eor(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = self.register_a ^ opcode_data;

        self.set_register_a(result);
    }
    
    fn ora(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        self.or_register_a(opcode_data);
    }

    fn asl(&mut self, memory_mode: MemoryAccessMode) -> u8 {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = opcode_data.wrapping_shl(1);
        
        if opcode_data >> 7  == 1{
            self.set_carry_status();
        } else {
            self.clear_carry_status();
        }

        self.write_opcode_result(self.program_pointer, result, memory_mode);
        self.update_status(result);

        return result;
    }

    fn lsr(&mut self, memory_mode: MemoryAccessMode) -> u8 {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = opcode_data >> 1;
        
        if opcode_data & 1 == 1 {
            self.set_carry_status();
        } else {
            self.clear_carry_status();
        }
        
        self.write_opcode_result(self.program_pointer, result, memory_mode);
        self.update_status(result);

        return result;
    }

    fn rol(&mut self, memory_mode: MemoryAccessMode) -> u8 {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        
        let mut result = opcode_data << 1;
        
        if self.status.contains(StatusFlags::CARRY) {
            result |= 1;
        }
        
        if opcode_data >> 7 == 1 {
            self.set_carry_status();
        } else {
            self.clear_carry_status();
        }
        
        self.write_opcode_result(self.program_pointer, result, memory_mode);
        self.update_negative_status(result);
        
        return result;
    }

    fn ror(&mut self, memory_mode: MemoryAccessMode) -> u8 {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        
        let mut result = opcode_data >> 1;
        
        if self.status.contains(StatusFlags::CARRY) {
            result |= 0b1000_0000;
        }
        
        if opcode_data & 1 == 1 {
            self.set_carry_status();
        } else {
            self.clear_carry_status();
        }
        
        self.write_opcode_result(self.program_pointer, result, memory_mode);
        self.update_negative_status(result);

        return result;
    }

    fn bit(&mut self, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        let result = self.register_a & opcode_data;

        if result == 0 {
            self.set_zero_status();
        } else {
            self.clear_zero_status();
        }

        self.status.set(StatusFlags::NEGATIVE, opcode_data & 0b10000000 > 0);
        self.status.set(StatusFlags::OVERFLOW, opcode_data & 0b01000000 > 0);
    }
    /* -bits */

    /* +interrupts */
    fn rti(&mut self, memory_mode: MemoryAccessMode) {
        info!("Returning from interrupt");

        let recovered_status = self.pop_stack_u8();
        
        debug!("Carefully recovering from interrupt; recovering status {:X}", recovered_status);
        let mut recovered_status = match StatusFlags::from_bits(recovered_status) {
            Some(status) => status,
            None => panic!("Well... no luck"),
        };
        recovered_status.remove(StatusFlags::BREAK);
        recovered_status.insert(StatusFlags::UNUSED);

        let recovered_counter = self.pop_stack_u16();
        debug!("Carefully recovering from interrupt; recovering program counter {}", recovered_counter);
        
        info!(
            "Recovering from interrupt, current status/PC: {:?}; {:X}; recovered: {:?}; {:X}",
            self.status, self.program_pointer, recovered_status, recovered_counter
        );

        self.status = recovered_status;
        self.program_pointer = recovered_counter as usize;
    }
    /* -interrupts */

    /* +branching */

    fn branch(&mut self, condition: bool, memory_mode: MemoryAccessMode) {
        if condition {
            debug!("Branched {:?}", memory_mode);

            self.bus.tick(1);
            
            let displacement = self.get_opcode_data_address(
                self.program_pointer, memory_mode, true
            ) as i8;

            debug!("Branching; PC {:X}", self.program_pointer);
            
            let program_pointer = (self.program_pointer as u16).wrapping_add(1);
            let jump_address = program_pointer.wrapping_add_signed(displacement as i16);
            
            debug!("Branching; New PC {:X}", jump_address);
            
            self.cross_page(program_pointer, jump_address);
            self.program_pointer = jump_address as usize;
        } else {
            debug!("Unsuccessfull branch");
        }
    }

    fn nop(&mut self, memory_mode: MemoryAccessMode) {
        self.program_pointer += 0;
    }

    fn jmp(&mut self, memory_mode: MemoryAccessMode) {
        let address = self.get_opcode_data_address(
            self.program_pointer, memory_mode, true
        );
    
        debug!("Carefully jumping from {:X} to {:X}, metadata {:?}", self.program_pointer, address, memory_mode);
    
        self.program_pointer = address as usize;
    }

    fn jsr(&mut self, opcode_metadata: &Opcode) {
        debug!("!!!JSR instruction start!!!");
        
        let address = self.get_opcode_data_address(
            self.program_pointer, opcode_metadata.memory_mode, true
        );
        
        debug!("Got address {:X}", address);
        
        let saved_pointer = (self.program_pointer as u16)
            .wrapping_add((opcode_metadata.length - 1) as u16) // read memory
            .wrapping_sub(1);
        
        debug!("Pushing PC to stack: actual PC {:X}; pushed PC + 2 -1 {:X}", self.program_pointer, saved_pointer);
        
        self.push_stack_u16(saved_pointer);
        self.program_pointer = address as usize; 
        
        debug!("!!!JSR instruction end!!!");
    }

    fn rts(&mut self, memory_mode: MemoryAccessMode) {
        let recovered_counter = self.pop_stack_u16();
        
        debug!("Carefully returning from subroutine; setting PC from {:X} to {:X}", self.program_pointer, recovered_counter.wrapping_add(1));
        
        self.program_pointer = recovered_counter.wrapping_add(1) as usize;
    }
  
    fn bcc(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(!self.status.contains(StatusFlags::CARRY), memory_mode);
    }

    fn bcs(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(self.status.contains(StatusFlags::CARRY), memory_mode);
    }

    fn beq(&mut self, memory_mode: MemoryAccessMode) {
        debug!("Beq! status {:?}", self.status);
        self.branch(self.status.contains(StatusFlags::ZERO), memory_mode);
    }

    fn bne(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(!self.status.contains(StatusFlags::ZERO), memory_mode);
    }

    fn bmi(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(self.status.contains(StatusFlags::NEGATIVE), memory_mode);
    }
    
    fn bpl(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(!self.status.contains(StatusFlags::NEGATIVE), memory_mode);
    }
    
    fn bvc(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(!self.status.contains(StatusFlags::OVERFLOW), memory_mode);
    }

    fn bvs(&mut self, memory_mode: MemoryAccessMode) {
        self.branch(self.status.contains(StatusFlags::OVERFLOW), memory_mode);
    }
    /* -branching */

    /* +statuses */
    fn sec(&mut self, memory_mode: MemoryAccessMode) {
        self.set_carry_status();
    }

    fn sed(&mut self, memory_mode: MemoryAccessMode) {
        self.set_decimal_mode_status();
    }

    fn sei(&mut self, memory_mode: MemoryAccessMode) {
        self.set_interrupt_disable_status();
    }

    fn clc(&mut self, memory_mode: MemoryAccessMode) {
        self.clear_carry_status();
    }

    fn cld(&mut self, memory_mode: MemoryAccessMode) {
        self.clear_decimal_mode_status();
    }

    fn cli(&mut self, memory_mode: MemoryAccessMode) {
        self.clear_interrupt_disable_status();
    }

    fn clv(&mut self, memory_mode: MemoryAccessMode) {
        self.clear_overflow_status();
    }

    /* -statuses */

    /* +stack */
    fn pha(&mut self, memory_mode: MemoryAccessMode) {
        self.push_stack_u8(self.register_a);
    }

    fn pla(&mut self, memory_mode: MemoryAccessMode) {
        let value = self.pop_stack_u8();
        // trace!("PLA before value: {:2X}; A: {:2X}", value, self.register_a);
        self.set_register_a(value);
        // trace!("PLA after value: {:2X}; A: {:2X}", value, self.register_a);
    }

    fn php(&mut self, memory_mode: MemoryAccessMode) {
        let mut status = self.status.clone();
        status.insert(StatusFlags::BREAK);
        status.insert(StatusFlags::UNUSED);
        
        self.push_stack_u8(status.bits());
    }
    
    fn plp(&mut self, memory_mode: MemoryAccessMode) {
        let old_status = self.pop_stack_u8();

        self.status = match StatusFlags::from_bits(old_status) {
            Some(status) => status,
            None => panic!("Pulled wrong status from stack {}", old_status),
        };
        self.status.remove(StatusFlags::BREAK);
        self.status.insert(StatusFlags::UNUSED);
    }
    /* -stack */

    fn compare(&mut self, with: u8, memory_mode: MemoryAccessMode) {
        let opcode_data = self.read_opcode_data(self.program_pointer, memory_mode);
        
        if with >= opcode_data {
            self.set_carry_status();
        } else {
            self.clear_carry_status();
        }
        debug!("CMP! {}; {}; {}", with, opcode_data, with.wrapping_sub(opcode_data));
        self.update_status(with.wrapping_sub(opcode_data));
    }

    fn cmp(&mut self, memory_mode: MemoryAccessMode) {
        self.compare(self.register_a, memory_mode);
    }

    fn cpx(&mut self, memory_mode: MemoryAccessMode) {
        self.compare(self.register_x, memory_mode);
    }

    fn cpy(&mut self, memory_mode: MemoryAccessMode) {
        self.compare(self.register_y, memory_mode);
    }
}