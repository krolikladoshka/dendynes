use crate::dendynes::cpu::processor::CPU;


const ZERO_PAGE: u16 = 0;
const PAGE_CROSSING_MASK: usize = 0xFF00;

#[derive(Debug, Copy, Hash, Clone, PartialEq, Eq)]
pub enum MemoryAccessMode {
    Implied,
    Immediate,
    Accumulator,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative(bool),
    Absolute,
    AbsoluteX(bool),
    AbsoluteY(bool),
    Indirect,
    IndirectX,
    IndirectY(bool),
}

impl MemoryAccessMode {
    pub fn can_page_cross(&self) -> bool {
        match self {
            Self::Relative(true) |
            Self::AbsoluteX(true) |
            Self::AbsoluteY(true) |
            Self::IndirectY(true)
            => {
                return true;
            }
            _ => {
                return false;
            }
        };
    }

    pub fn page_crossed(base: usize, address: usize) -> bool {
        return (base & PAGE_CROSSING_MASK) != (address & PAGE_CROSSING_MASK);
    }

    pub fn map_address(&self, cpu: &mut CPU, address: u16) -> u16 {
        // rewrite it to fetch opcode arg 
        match self {
            MemoryAccessMode::Implied => {
                return cpu.program_pointer as u16;
            },
            MemoryAccessMode::Immediate => {
                return (cpu.program_pointer + 1) as u16;
            },
            MemoryAccessMode::Accumulator => {
                return cpu.register_a as u16;
            },
            MemoryAccessMode::ZeroPage => {
                return ZERO_PAGE + address;
            },
            MemoryAccessMode::ZeroPageX => {
                return (((ZERO_PAGE + address) as u8).wrapping_add(cpu.register_x)) as u16;
            },
            MemoryAccessMode::ZeroPageY => {
                return (((ZERO_PAGE + address) as u8).wrapping_add(cpu.register_y)) as u16;
            },
            MemoryAccessMode::Relative(crossed_page) => {
                let offset = &0xffu8; // TODO: Placeholder

                let result = if (*offset & (1 << 7)) >> 7 == 1 {
                    // negative
                    (cpu.program_pointer - ((!(*offset) + 1) as usize)) as u16
                } else {
                    (cpu.program_pointer + *offset as usize) as u16
                };

                if *crossed_page {
                    cpu.bus.tick(1);
                }

                return result;
            },
            MemoryAccessMode::Absolute => {
                return 0;
            }
            _ => panic!(""),
        }
    }
}