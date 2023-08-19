pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;

pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub struct Display {
    pub screen: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display {
    pub fn new() -> Display {
        Display {
            screen: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    pub fn draw(&mut self, point: Point, sprite: &[u8]) -> bool {
        let mut change = false;

        for (index, byte) in sprite.iter().enumerate() {
            let y = (point.y + index) % DISPLAY_HEIGHT;

            // each sprite is 1 byte (8 bits) long, hence why we iterate 8 times
            for index in 0..8 {
                let x = (point.x + index) % DISPLAY_WIDTH;

                let block = self.screen[y][x];
                let bit = (byte >> (7 - index)) & 1;

                let new_block = (bit ^ block as u8) != 0;

                self.screen[y][x] = new_block;

                if block && !new_block {
                    change = true;
                }
            }
        }

        change
    }

    pub fn clear(&mut self) {
        self.screen = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ****
    // *  *
    // *  *
    // *  *
    // ****
    const ZERO: [u8; 5] = [0xF0, 0x90, 0x90, 0x90, 0xF0];

    #[test]
    fn new_is_empty() {
        let display = Display::new();

        for index in 0..display.screen.len() {
            for block in display.screen[index] {
                assert!(!block);
            }
        }
    }

    #[test]
    fn drew_sprite() {
        let mut display = Display::new();

        display.draw(Point { x: 0, y: 0 }, &ZERO);

        assert_eq!(display.screen[0][..4], [true, true, true, true]);
        assert_eq!(display.screen[1][..4], [true, false, false, true]);
        assert_eq!(display.screen[2][..4], [true, false, false, true]);
        assert_eq!(display.screen[3][..4], [true, false, false, true]);
        assert_eq!(display.screen[4][..4], [true, true, true, true]);
    }

    #[test]
    fn clears_screen() {
        let mut display = Display::new();

        display.draw(Point { x: 0, y: 0 }, &ZERO);

        display.clear();

        for index in 0..display.screen.len() {
            for block in display.screen[index] {
                assert!(!block);
            }
        }
    }
}
