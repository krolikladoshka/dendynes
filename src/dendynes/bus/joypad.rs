use bitflags::bitflags;


bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct JoypadButtons: u8 {
        const RIGHT  = 0b0000_0001;
        const LEFT   = 0b0000_0010;
        const DOWN   = 0b0000_0100;
        const UP     = 0b0000_1000;
        const START  = 0b0001_0000;
        const SELECT = 0b0010_0000;
        const B      = 0b0100_0000;
        const A      = 0b1000_0000;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Joypad {
    pub buttons_pressed: JoypadButtons,
    state: u8, 
}

impl Joypad {
    pub fn new() -> Joypad {
        return Joypad {
            buttons_pressed: JoypadButtons::empty(),
            state: 0,
        };
    }

    pub fn read(&mut self) -> u8 {
        let data = (self.state & 0x80 > 0) as u8;
        self.state <<= 1;

        return data;
    }

    pub fn write(&mut self) {
        self.state = self.buttons_pressed.bits();
    }

    pub fn press_button(&mut self, button: JoypadButtons) {
        self.buttons_pressed.insert(button);
    }

    pub fn release_button(&mut self, button: JoypadButtons) {
        self.buttons_pressed.remove(button);
    }
}
