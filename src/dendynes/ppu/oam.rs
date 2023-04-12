#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OamSprite {
    pub y: u8,
    pub id: u8,
    pub attribute: u8,
    pub x: u8,
}

impl OamSprite {
    pub fn new_empty() -> Self {
        return OamSprite {
            y: 0,
            id: 0,
            attribute: 0,
            x: 0,
        };
    }

    pub fn new(x: u8, y: u8, id: u8, attribute: u8) -> Self {
        return OamSprite {
            y: y,
            id: id,
            attribute: attribute,
            x: x,
        };
    }

    pub fn set_attribute_by_address(&mut self, address: u8, value: u8) {
        match address % 4 {
            0 => {
                self.y = value;
            },
            1 => {
                self.id = value;
            },
            2 => {
                self.attribute = value;
            },
            3 => {
                self.x = value;
            },
            _ => {
                panic!("Can't be!");
            }
        }
    }

    pub fn get_attribute_by_address(&self, address: u8) -> u8 {
        match address % 4 {
            0 => {
                return self.y;
            },
            1 => {
                return self.id;
            },
            2 => {
                return self.attribute;
            },
            3 => {
                return self.x;
            },
            _ => {
                panic!("Can't be!");
            }
        }
    }
}