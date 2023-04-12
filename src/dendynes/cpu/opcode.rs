use std::collections::HashMap;

use enum_primitive_derive::Primitive;
use lazy_static::lazy_static;

use crate::dendynes::memory::accessing_mode::MemoryAccessMode;


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Primitive, Debug, Copy, Clone)]
pub enum OpcodeType {
    // unimplemented
    Brk = 0x0,
    
    // implemented
    AdcI = 0x69,
    AdcZp = 0x65,
    AdcZpx = 0x75,
    AdcA = 0x6D,
    AdcAx = 0x7D,
    AdcAy = 0x79,
    AdcIx = 0x61,
    AdcIy = 0x71,

    // implemented
    AndI = 0x29,
    AndZp = 0x25,
    AndZpx = 0x35,
    AndA = 0x2D,
    AndAx = 0x3D,
    AndAy = 0x39,
    AndIx = 0x21,
    AndIy = 0x31,

    // implemented
    AslAcc = 0x0A,
    AslZp = 0x06,
    AslZpx = 0x16,
    AslA = 0x0E,
    AslAx = 0x1E,

    LsrAcc = 0x4A,
    LsrZp = 0x46,
    LsrZpx = 0x56,
    LsrA = 0x4E,
    LsrAx = 0x5e,

    // implemented
    Bcc = 0x90,
    Bcs = 0xB0,
    Beq = 0xF0,
    
    // implemented
    BitZp = 0x24,
    BitA = 0x2C,

    // implemented
    Bmi = 0x30,

    // implemented
    Bne = 0xD0,
    Bpl = 0x10,
    Bvc = 0x50,
    Bvs = 0x70,

    // implemented
    Clc = 0x18,
    Cld = 0xD8,
    Cli = 0x58,
    Clv = 0xB8,
    
    // implemented
    CmpI = 0xC9,
    CmpZp = 0xC5,
    CmpZpx = 0xD5,
    CmpA = 0xCD,
    CmpAx = 0xDD,
    CmpAy = 0xD9,
    CmpIx = 0xC1,
    CmpIy = 0xD1,
    
    CpxI = 0xE0,
    CpxZp = 0xE4,
    CpxA = 0xEC,

    CpyI = 0xC0,
    CpyZp = 0xC4,
    CpyA = 0xCC,
    
    // implemented
    DecZp = 0xC6,
    DecZpx = 0xD6,
    DecA = 0xCE,
    DecAx = 0xDE,

    Dex = 0xCA,
    Dey = 0x88,

    IncZp = 0xE6,
    IncZpx = 0xF6,
    IncA = 0xEE,
    IncAx = 0xFE,

    Inx = 0xE8,
    Iny = 0xC8,

    // implemented
    EorI = 0x49,
    EorZp = 0x45,
    EorZpx = 0x55,
    EorA = 0x4D,
    EorAx = 0x5D,
    EorAy = 0x59,
    EorIx = 0x41,
    EorIy = 0x51,

    //implemented
    JmpA = 0x4C,
    JmpInd = 0x6C,

    Jsr = 0x20,

    // implemented
    LdaI = 0xA9,
    LdaZp = 0xA5,
    LdaZpx = 0xB5,
    LdaA = 0xAD,
    LdaAx = 0xBD,
    LdaAy = 0xB9,
    LdaIx = 0xA1,
    LdaIy = 0xB1,

    LdxI = 0xA2,
    LdxZp = 0xA6,
    LdxZpy = 0xB6,
    LdxA = 0xAE,
    LdxAy = 0xBE,

    LdyI = 0xA0,
    LdyZp = 0xA4,
    LdyZpx = 0xB4,
    LdyA = 0xAC,
    LdyAx = 0xBC,

    // implemented
    NopHlt = 0x02,
    Nop = 0xEA,
    Nop1 = 0x1A,
    Nop2 = 0x3A,
    Nop3 = 0x5A,
    Nop4 = 0x7A,
    Nop5 = 0xDA,
    Nop6 = 0xFA,

    // implemented
    OraI = 0x09,
    OraZp = 0x05,
    OraZpx = 0x15,
    OraA = 0x0D,
    OraAx = 0x1D,
    OraAy = 0x19,
    OraIx = 0x01,
    OraIy = 0x11,

    // implemented
    Pha = 0x48,
    Php = 0x08,
    Pla = 0x68,
    Plp = 0x28,

    // implemented
    RolAcc = 0x2A,
    RolZp = 0x26,
    RolZpx = 0x36,
    RolA = 0x2E,
    RolAx = 0x3E,

    RorAcc = 0x6A,
    RorZp = 0x66,
    RorZpx = 0x76,
    RorA = 0x6E,
    RorAx = 0x7E,

    // implemented
    Rti = 0x40,
    Rts = 0x60,
    
    // implemented
    SbcI = 0xE9,
    SbcZp = 0xE5,
    SbcZpx = 0xF5,
    SbcA = 0xED,
    SbcAx = 0xFD,
    SbcAy = 0xF9,
    SbcIx = 0xE1,
    SbcIy = 0xF1,

    // implemented
    Sec = 0x38,
    Sed = 0xF8,
    Sei = 0x78,

    // implemented
    StaZp = 0x85,
    StaZpx = 0x95,
    StaA = 0x8D,
    StaAx = 0x9D,
    StaAy = 0x99,
    StaIx = 0x81,
    StaIy = 0x91,

    StxZp = 0x86,
    StxZpy = 0x96,
    StxA = 0x8e,

    StyZp = 0x84,
    StyZpx = 0x94,
    StyA = 0x8C,

    // implemented
    Tax = 0xAA,
    Tay = 0xA8,
    Tsx = 0xBA,
    Txa = 0x8A,
    Txs = 0x9A,
    Tya = 0x98,
    
    // unofficial

    // implemented
    AsoZp = 0x07,
    AsoZpx = 0x17,
    AsoA = 0x0F,
    AsoAx = 0x1F,
    AsoAy = 0x1B,
    AsoIx = 0x03,
    AsoIy = 0x13,

    RlaZp = 0x27,
    RlaZpx = 0x37,
    RlaA = 0x2F,
    RlaAx = 0x3F,
    RlaAy = 0x3B,
    RlaIx = 0x23, 
    RlaIy = 0x33,

    LseZp = 0x47,
    LseZpx = 0x57,
    LseA = 0x4F,
    LseAx = 0x5f,
    LseAy = 0x5B,
    LseIx = 0x43,
    LseIy = 0x53,

    RraZp = 0x67,
    RraZpx = 0x77,
    RraA = 0x6F,
    RraAx = 0x7F,
    RraAy = 0x7B,
    RraIx = 0x63,
    RraIy = 0x73,

    AxsZp = 0x87,
    AxsZpy = 0x97,
    AxsA = 0x8F,
    AxsIx = 0x83,

    LaxZp = 0xA7,
    LaxZpy = 0xB7,
    LaxA = 0xAF,
    LaxAy = 0xBF,
    LaxIx = 0xA3,
    LaxIy = 0xB3,

    Skb = 0x80,
    Skb1 = 0x82,
    Skb2 = 0xC2,
    Skb3 = 0xE2,
    Skb4 = 0x04,
    Skb5 = 0x14,
    Skb6 = 0x34,
    Skb7 = 0x44,
    Skb8 = 0x54,
    Skb9 = 0x64,
    Skb10 = 0x74,
    Skb11 = 0xD4,
    Skb12 = 0xF4,
    Skb13 = 0x89,
    
    Skb14 = 0x0C,
    Skb15 = 0x1C,
    Skb16 = 0x3C,
    Skb17 = 0x5C,
    Skb18 = 0x7C,
    Skb19 = 0xDC,
    Skb20 = 0xFC,

    Sbc1I = 0xEb,

    DcmZp = 0xC7,
    DcmZpx = 0xD7,
    DcmA = 0xCF,
    DcmAx = 0xDF,
    DcmAy = 0xDB,
    DcmIx = 0xC3,
    DcmIy = 0xD3,

    InsZp = 0xE7,
    InsZpx = 0xF7,
    InsA = 0xEF,
    InsAx = 0xFF,
    InsAy = 0xFb,
    InsIx = 0xE3,
    InsIy = 0xF3,

    AxaAy = 0x9F,
    AxaIy = 0x93,

    Say = 0x9C,
}


#[derive(Debug)]
pub struct Opcode {
    pub code: OpcodeType,
    pub name: &'static str,
    pub length: u8,
    pub cycles: u8,
    pub memory_mode: MemoryAccessMode,
}

impl Opcode {
    pub fn new(code: OpcodeType, name: &'static str, length: u8, cycles: u8, memory_mode: MemoryAccessMode) -> Self {
        return Opcode {
            code: code,
            name: name,
            length: length,
            cycles: cycles,
            memory_mode: memory_mode,
        };
    }
}


lazy_static! {
    pub static ref CPU_OPCODES: Vec<Opcode> = vec![
        Opcode::new(OpcodeType::Brk, "BRK", 1, 7, MemoryAccessMode::Implied),
       
        Opcode::new(OpcodeType::AdcI, "ADC", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::AdcZp, "ADC", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::AdcZpx, "ADC", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::AdcA, "ADC", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::AdcAx, "ADC", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::AdcAy, "ADC", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::AdcIx, "ADC", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::AdcIy, "ADC", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::AndI, "AND", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::AndZp, "AND", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::AndZpx, "AND", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::AndA, "AND", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::AndAx, "AND", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::AndAy, "AND", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::AndIx, "AND", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::AndIy, "AND", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::AslAcc, "ASL", 1, 2, MemoryAccessMode::Accumulator),
        Opcode::new(OpcodeType::AslZp, "ASL", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::AslZpx, "ASL", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::AslA, "ASL", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::AslAx, "ASL", 3, 7, MemoryAccessMode::AbsoluteX(false)),

        Opcode::new(OpcodeType::LsrAcc, "LSR", 1, 2, MemoryAccessMode::Accumulator),
        Opcode::new(OpcodeType::LsrZp, "LSR", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::LsrZpx, "LSR", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::LsrA, "LSR", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::LsrAx, "LSR", 3, 7, MemoryAccessMode::AbsoluteX(false)),

        Opcode::new(OpcodeType::Bcc, "BCC", 2, 2, MemoryAccessMode::Relative(true)),
        Opcode::new(OpcodeType::Bcs, "BCS", 2, 2, MemoryAccessMode::Relative(true)),
        Opcode::new(OpcodeType::Beq, "BEQ", 2, 2, MemoryAccessMode::Relative(true)),

        Opcode::new(OpcodeType::BitZp, "BIT", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::BitA, "BIT", 3, 4, MemoryAccessMode::Absolute),

        Opcode::new(OpcodeType::Bmi, "BMI", 2, 2, MemoryAccessMode::Relative(true)),
        Opcode::new(OpcodeType::Bne, "BNE", 2, 2, MemoryAccessMode::Relative(true)),
        Opcode::new(OpcodeType::Bpl, "BPL", 2, 2, MemoryAccessMode::Relative(true)),
        Opcode::new(OpcodeType::Bvc, "BVC", 2, 2, MemoryAccessMode::Relative(true)),
        Opcode::new(OpcodeType::Bvs, "BVS", 2, 2, MemoryAccessMode::Relative(true)),

        Opcode::new(OpcodeType::Clc, "CLC", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Cld, "CLD", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Cli, "CLI", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Clv, "CLV", 1, 2, MemoryAccessMode::Implied),

        Opcode::new(OpcodeType::CmpI, "CMP", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::CmpZp, "CMP", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::CmpZpx, "CMP", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::CmpA, "CMP", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::CmpAx, "CMP", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::CmpAy, "CMP", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::CmpIx, "CMP", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::CmpIy, "CMP", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::CpxI, "CPX", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::CpxZp, "CPX", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::CpxA, "CPX", 3, 4, MemoryAccessMode::Absolute),

        Opcode::new(OpcodeType::CpyI, "CPY", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::CpyZp, "CPY", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::CpyA, "CPY", 3, 4, MemoryAccessMode::Absolute),

        Opcode::new(OpcodeType::DecZp, "DEC", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::DecZpx, "DEC", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::DecA, "DEC", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::DecAx, "DEC", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        
        Opcode::new(OpcodeType::Dex, "DEX", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Dey, "DEY", 1, 2, MemoryAccessMode::Implied),

        Opcode::new(OpcodeType::EorI, "EOR", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::EorZp, "EOR", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::EorZpx, "EOR", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::EorA, "EOR", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::EorAx, "EOR", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::EorAy, "EOR", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::EorIx, "EOR", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::EorIy, "EOR", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::IncZp, "INC", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::IncZpx, "INC", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::IncA, "INC", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::IncAx, "INC", 3, 7, MemoryAccessMode::AbsoluteX(false)),

        Opcode::new(OpcodeType::Inx, "INX", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Iny, "INY", 1, 2, MemoryAccessMode::Implied),

        Opcode::new(OpcodeType::JmpA, "JMP", 3, 3, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::JmpInd, "JMP", 3, 5, MemoryAccessMode::Indirect),

        Opcode::new(OpcodeType::Jsr, "JSR", 3, 6, MemoryAccessMode::Absolute),

        Opcode::new(OpcodeType::LdaI, "LDA", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::LdaZp, "LDA", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::LdaZpx, "LDA", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::LdaA, "LDA", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::LdaAx, "LDA", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::LdaAy, "LDA", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::LdaIx, "LDA", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::LdaIy, "LDA", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::LdxI, "LDX", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::LdxZp, "LDX", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::LdxZpy, "LDX", 2, 4, MemoryAccessMode::ZeroPageY),
        Opcode::new(OpcodeType::LdxA, "LDX", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::LdxAy, "LDX", 3, 4, MemoryAccessMode::AbsoluteY(true)),

        Opcode::new(OpcodeType::LdyI, "LDY", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::LdyZp, "LDY", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::LdyZpx, "LDY", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::LdyA, "LDY", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::LdyAx, "LDY", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        
        Opcode::new(OpcodeType::NopHlt, "HLT", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop, "NOP", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop1, "NOP1", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop2, "NOP2", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop3, "NOP3", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop4, "NOP4", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop5, "NOP5", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Nop6, "NOP6", 1, 2, MemoryAccessMode::Implied),


        Opcode::new(OpcodeType::OraI, "ORA", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::OraZp, "ORA", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::OraZpx, "ORA", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::OraA, "ORA", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::OraAx, "ORA", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::OraAy, "ORA", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::OraIx, "ORA", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::OraIy, "ORA", 2, 5, MemoryAccessMode::IndirectY(true)),
        
        Opcode::new(OpcodeType::Pha, "PHA", 1, 3, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Php, "PHP", 1, 3, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Pla, "PLA", 1, 4, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Plp, "PLP", 1, 4, MemoryAccessMode::Implied),
        
        Opcode::new(OpcodeType::RolAcc, "ROL", 1, 2, MemoryAccessMode::Accumulator),
        Opcode::new(OpcodeType::RolZp, "ROL", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::RolZpx, "ROL", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::RolA, "ROL", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::RolAx, "ROL", 3, 7, MemoryAccessMode::AbsoluteX(false)),

        Opcode::new(OpcodeType::RorAcc, "ROR", 1, 2, MemoryAccessMode::Accumulator),
        Opcode::new(OpcodeType::RorZp, "ROR", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::RorZpx, "ROR", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::RorA, "ROR", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::RorAx, "ROR", 3, 7, MemoryAccessMode::AbsoluteX(false)),

        Opcode::new(OpcodeType::Rti, "RTI", 1, 6, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Rts, "RTS", 1, 6, MemoryAccessMode::Implied),

        Opcode::new(OpcodeType::SbcI, "SBC", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::SbcZp, "SBC", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::SbcZpx, "SBC", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::SbcA, "SBC", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::SbcAx, "SBC", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::SbcAy, "SBC", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::SbcIx, "SBC", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::SbcIy, "SBC", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::Sec, "SEC", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Sed, "SED", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Sei, "SEI", 1, 2, MemoryAccessMode::Implied),

        Opcode::new(OpcodeType::StaZp, "STA", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::StaZpx, "STA", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::StaA, "STA", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::StaAx, "STA", 3, 5, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::StaAy, "STA", 3, 5, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::StaIx, "STA", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::StaIy, "STA", 2, 6, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::StxZp, "STX", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::StxZpy, "STX", 2, 4, MemoryAccessMode::ZeroPageY),
        Opcode::new(OpcodeType::StxA, "STX", 3, 4, MemoryAccessMode::Absolute),

        Opcode::new(OpcodeType::StyZp, "STY", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::StyZpx, "STY", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::StyA, "STX", 3, 4, MemoryAccessMode::Absolute),

        Opcode::new(OpcodeType::Tax, "TAX", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Tay, "TAY", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Tsx, "TSX", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Txa, "TXA", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Txs, "TXS", 1, 2, MemoryAccessMode::Implied),
        Opcode::new(OpcodeType::Tya, "TYA", 1, 2, MemoryAccessMode::Implied),

        // unofficial
        Opcode::new(OpcodeType::AsoZp, "ASO", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::AsoZpx, "ASO", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::AsoA, "ASO", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::AsoAx, "ASO", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::AsoAy, "ASO", 3, 7, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::AsoIx, "ASO", 2, 8, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::AsoIy, "ASO", 2, 8, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::RlaZp, "RLA", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::RlaZpx, "RLA", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::RlaA, "RLA", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::RlaAx, "RLA", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::RlaAy, "RLA", 3, 7, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::RlaIx, "RLA", 2, 8, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::RlaIy, "RLA", 2, 8, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::LseZp, "LSE", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::LseZpx, "LSE", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::LseA, "LSE", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::LseAx, "LSE", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::LseAy, "LSE", 3, 7, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::LseIx, "LSE", 2, 8, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::LseIy, "LSE", 2, 8, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::RraZp, "RRA", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::RraZpx, "RRA", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::RraA, "RRA", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::RraAx, "RRA", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::RraAy, "RRA", 3, 7, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::RraIx, "RRA", 2, 8, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::RraIy, "RRA", 2, 8, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::AxsZp, "AXS", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::AxsZpy, "AXS", 2, 4, MemoryAccessMode::ZeroPageY),
        Opcode::new(OpcodeType::AxsA, "AXS", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::AxsIx, "AXS", 2, 6, MemoryAccessMode::IndirectX),

        Opcode::new(OpcodeType::LaxZp, "LAX", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::LaxZpy, "LAX", 2, 4, MemoryAccessMode::ZeroPageY),
        Opcode::new(OpcodeType::LaxA, "LAX", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::LaxAy, "LAX", 3, 4, MemoryAccessMode::AbsoluteY(true)),
        Opcode::new(OpcodeType::LaxIx, "LAX", 2, 6, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::LaxIy, "LAX", 2, 5, MemoryAccessMode::IndirectY(true)),

        Opcode::new(OpcodeType::Skb, "SKB", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::Skb1, "SKB1", 2, 2, MemoryAccessMode::Immediate),

        Opcode::new(OpcodeType::Skb2, "SKB2", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::Skb3, "SKB3", 2, 2, MemoryAccessMode::Immediate),
        Opcode::new(OpcodeType::Skb13, "SKB13", 2, 2, MemoryAccessMode::Immediate),

        Opcode::new(OpcodeType::Skb4, "SKB4", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::Skb5, "SKB5", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::Skb6, "SKB6", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::Skb7, "SKB7", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::Skb8, "SKB8", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::Skb9, "SKB9", 2, 3, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::Skb10, "SKB10", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::Skb11, "SKB11", 2, 4, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::Skb12, "SKB12", 2, 4, MemoryAccessMode::ZeroPageX),

        Opcode::new(OpcodeType::Skb14, "SKB14", 3, 4, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::Skb15, "SKB15", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::Skb16, "SKB16", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::Skb17, "SKB17", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::Skb18, "SKB18", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::Skb19, "SKB19", 3, 4, MemoryAccessMode::AbsoluteX(true)),
        Opcode::new(OpcodeType::Skb20, "SKB20", 3, 4, MemoryAccessMode::AbsoluteX(true)),

        Opcode::new(OpcodeType::Sbc1I, "SBC1", 2, 2, MemoryAccessMode::Immediate),

        Opcode::new(OpcodeType::DcmZp, "DCM", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::DcmZpx, "DCM", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::DcmA, "DCM", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::DcmAx, "DCM", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::DcmAy, "DCM", 3, 7, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::DcmIx, "DCM", 2, 8, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::DcmIy, "DCM", 2, 8, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::InsZp, "INS", 2, 5, MemoryAccessMode::ZeroPage),
        Opcode::new(OpcodeType::InsZpx, "INS", 2, 6, MemoryAccessMode::ZeroPageX),
        Opcode::new(OpcodeType::InsA, "INS", 3, 6, MemoryAccessMode::Absolute),
        Opcode::new(OpcodeType::InsAx, "INS", 3, 7, MemoryAccessMode::AbsoluteX(false)),
        Opcode::new(OpcodeType::InsAy, "INS", 3, 7, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::InsIx, "INS", 2, 8, MemoryAccessMode::IndirectX),
        Opcode::new(OpcodeType::InsIy, "INS", 2, 8, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::AxaAy, "AXA", 3, 5, MemoryAccessMode::AbsoluteY(false)),
        Opcode::new(OpcodeType::AxaIy, "AXA", 2, 6, MemoryAccessMode::IndirectY(false)),

        Opcode::new(OpcodeType::Say, "SAY", 3, 5, MemoryAccessMode::AbsoluteX(false)),
    ];

    pub static ref OPCODES_MAP: HashMap<OpcodeType, &'static Opcode> = {
        let mut map = HashMap::new();
        
        for opcode in &*CPU_OPCODES {
            map.insert(opcode.code, opcode);
        }
        
        return map
    };
}