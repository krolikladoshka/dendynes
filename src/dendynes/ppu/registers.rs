use bitflags::bitflags;


pub const OPEN_BUS_REGISTER_MASK: u8 = (1 << 5) - 1;
pub const VRAM_ADDRESS_INC: u16 = 1;
pub const VRAM_ADDRESS_INC_VERTICAL_MODE: u16 = 32;
pub const PPU_ADDRESS_MIRROR_MASK: u16 = 0x3FFF;


bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Controller: u8 {
        const NAMETABLE_X                = 0b00000001;
        const NAMETABLE_Y                = 0b00000010;
        const VRAM_ADDRESS_INCREMENT     = 0b00000100;
        const SPRITE_PATTERN_ADDRESS     = 0b00001000;
        const BACKROUND_PATTERN_ADDRESS  = 0b00010000;
        const SPRITE_SIZE                = 0b00100000;
        const MASTER_SLAVE_SELECT        = 0b01000000;
        const GENERATE_NMI               = 0b10000000;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Mask: u8 {
        const GREYSCALE                = 0b0000_0001;
        const LEFTMOST_SHOW_BACKGROUND = 0b0000_0010;
        const LEFTMOST_SHOW_SPRITES    = 0b0000_0100;
        const SHOW_BACKGROUND          = 0b0000_1000;
        const SHOW_SPRITES             = 0b0001_0000;
        const EMPHASIZE_RED            = 0b0010_0000;
        const EMPHASIZE_GREEN          = 0b0100_0000;
        const EMPHASIZE_BLUE           = 0b1000_0000;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Status: u8 {
        const OPENBUS0        = 0b0000_0001;
        const OPENBUS1        = 0b0000_0010;
        const OPENBUS2        = 0b0000_0100;
        const OPENBUS3        = 0b0000_1000;
        const OPENBUS4        = 0b0001_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const VERTICAL_BLANK  = 0b1000_0000;
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LoopyRegister {
    coarse_x: u8,
    coarse_y: u8, 
    pub nametable_x: u8,
    pub nametable_y: u8,
    fine_y: u8,
}

impl LoopyRegister {
    pub fn new() -> Self {
        return LoopyRegister {
            coarse_x: 0,
            coarse_y: 0,
            nametable_x: 0,
            nametable_y: 0,
            fine_y: 0,
        };
    }

    pub fn flip_nametable_x(&mut self) {
        if self.nametable_x == 1 {
            self.nametable_x = 0;
        } else {
            self.nametable_x = 1;
        }
    }

    pub fn flip_nametable_y(&mut self) {
        if self.nametable_y == 1 {
            self.nametable_y = 0;
        } else {
            self.nametable_y = 1;
        }
    }

    pub fn increment_coarse_x(&mut self) {
        if self.coarse_x() == 31 {
            self.set_coarse_x(0);
            self.flip_nametable_x();
        } else {
            self.set_coarse_x(self.coarse_x() + 1);
        }
    }

    pub fn increment_coarse_y(&mut self) {
        if self.fine_y() < 7 {
            self.set_fine_y(self.fine_y + 1);
        } else {
            self.set_fine_y(0);

            if self.coarse_y() == 29 {
                self.set_coarse_y(0);
                self.flip_nametable_y();
            } else if self.coarse_y() == 31 {
                self.set_coarse_y(0);
            } else {
                self.set_coarse_y(self.coarse_y() + 1);
            }
        }
    }

    pub fn set_coarse_x(&mut self, value: u8) {
        self.coarse_x = value;
    }

    pub fn set_coarse_y(&mut self, value: u8) {
        self.coarse_y = value;
    }

    pub fn coarse_x(&self) -> u8 {
        return self.coarse_x & 0b11111;
    }
    
    pub fn coarse_y(&self) -> u8 {
        return self.coarse_y & 0b11111;
    }

    pub fn fine_y(&self) -> u8 {
        return self.fine_y & 0b111;
    }

    pub fn set_fine_y(&mut self, value: u8) {
        self.fine_y = value & 0b111;
    }

    pub fn address(&self) -> u16 {
        return ((self.fine_y() as u16) << 12) |
               ((self.nametable_y as u16) << 11) |
               ((self.nametable_x as u16) << 10) |
               ((self.coarse_y() as u16) << 5) |
               self.coarse_x() as u16;
    }

    pub fn vram_address(&self) -> u16 {
        return self.address() & 0xFFF;
    }

    pub fn next_tile_attribute_address(&self) -> u16 {
        return ((self.nametable_y as u16) << 11) |
               ((self.nametable_x as u16) << 10) |
               (((self.coarse_y() as u16) >> 2) << 3) |
               ((self.coarse_x() as u16) >> 2);
    }

    pub fn transfer_from(&mut self, other: &Self) {
        self.coarse_x = other.coarse_x;
        self.coarse_y = other.coarse_y;
        self.nametable_x = other.nametable_x;
        self.nametable_y = other.nametable_y;
        self.fine_y = other.fine_y;
    }

    pub fn transfer_x_from(&mut self, other: &Self) {
        self.nametable_x = other.nametable_x;
        self.coarse_x = other.coarse_x;
    }

    pub fn transfer_y_from(&mut self, other: &Self) {
        self.fine_y = other.fine_y;
        self.nametable_y = other.nametable_y;
        self.coarse_y = other.coarse_y;
    }

    pub fn set(&mut self, value: u16) {
        self.coarse_x = (value & 0b11111) as u8;
        self.coarse_y = ((value >> 5) & 0b11111) as u8;
        self.nametable_x = ((value >> 10) & 1) as u8;
        self.nametable_y = ((value >> 11) & 1) as u8;
        self.fine_y = ((value >> 12) & 0b111) as u8;
    }

    pub fn increment(&mut self, vertical_mode: bool) {
        let mut new_address = self.address();

        if vertical_mode {
            new_address = new_address.wrapping_add(VRAM_ADDRESS_INC_VERTICAL_MODE);
        } else {
            new_address = new_address.wrapping_add(VRAM_ADDRESS_INC);
        }

        self.set(new_address);
    }
}


impl Controller {
    pub fn get_sprite_size(&self) -> u8 {
        if self.contains(Self::SPRITE_SIZE) {
            return 16;
        } else {
            return 8;
        }
    }
}

impl Status {
    pub fn read(&mut self, open_bus: u8) -> u8 {
        let result = self.bits() | (open_bus & OPEN_BUS_REGISTER_MASK);
        
        self.remove(Self::VERTICAL_BLANK);

        return result;
    }

    pub fn reset(&mut self) {
        self.remove(Status::SPRITE_OVERFLOW);
        self.remove(Status::SPRITE_ZERO_HIT);
        self.remove(Status::VERTICAL_BLANK);
    }
}

impl Mask {
    pub fn is_render_enabled(&self) -> bool {
        return self.is_background_enabled() || self.is_foreground_enabled();
    }
    
    pub fn is_background_enabled(&self) -> bool {
        return self.contains(Mask::SHOW_BACKGROUND);
    }

    pub fn is_foreground_enabled(&self) -> bool {
        return self.contains(Mask::SHOW_SPRITES);
    }
}
