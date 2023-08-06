use breakout_common::fixedpoint::FixedPoint;
use atsamd21g::Peripherals;

use crate::oled::{DisplayCommand, DisplayInterface};


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Vec2 {
    pub x: FixedPoint,
    pub y: FixedPoint,
}
impl Vec2 {
    #[inline]
    pub fn flip_x(&mut self) {
        self.x = -self.x;
    }

    #[inline]
    pub fn flip_y(&mut self) {
        self.y = -self.y;
    }
}


#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Ball {
    pub position: Vec2,
    pub velocity: Vec2,
}


// the display is 96x96
//
// by height:
// 8px score digits
// 1px buffer
// 1px playfield border
// 85px playfield (with 1px border on each side)
// 1px playfield border


const DISPLAY_WIDTH: usize = 96;
const DISPLAY_HEIGHT: usize = 96;
const PLAYFIELD_WIDTH: FixedPoint = FixedPoint::new_integer(94);
const PLAYFIELD_HEIGHT: FixedPoint = FixedPoint::new_integer(85);
const PLAYFIELD_LEFT: usize = 1;
const PLAYFIELD_TOP: usize = 10;

const BYTES_PER_PIXEL: usize = 2; // R5:G6:B5 encoding
const DISPLAY_ROW_BYTES: usize = DISPLAY_WIDTH * BYTES_PER_PIXEL;
const DISPLAY_BYTES: usize = DISPLAY_HEIGHT * DISPLAY_ROW_BYTES;


pub(crate) struct Playfield {
    pub ball: Ball,
}
impl Playfield {
    pub fn new() -> Self {
        Self {
            ball: Ball {
                position: Vec2 { x: FixedPoint::zero(), y: FixedPoint::zero() },
                velocity: Vec2 {
                    // approximation to 4x the 45-degree unit vector (1/sqrt(2))
                    x: FixedPoint::new_raw(4 * 0b1011_0110),
                    y: FixedPoint::new_raw(4 * 0b1011_0110),
                },
            }
        }
    }

    fn advance_ball(&mut self) {
        self.ball.position.x += self.ball.velocity.x;
        self.ball.position.y += self.ball.velocity.y;

        if self.ball.position.x < FixedPoint::zero() {
            self.ball.position.x = FixedPoint::zero();
            self.ball.velocity.flip_x();
        }
        if self.ball.position.x >= PLAYFIELD_WIDTH {
            self.ball.position.x = PLAYFIELD_WIDTH - FixedPoint::one();
            self.ball.velocity.flip_x();
        }
        if self.ball.position.y < FixedPoint::zero() {
            self.ball.position.y = FixedPoint::zero();
            self.ball.velocity.flip_y();
        }
        if self.ball.position.y >= PLAYFIELD_HEIGHT {
            self.ball.position.y = PLAYFIELD_HEIGHT - FixedPoint::one();
            self.ball.velocity.flip_y();
        }
    }

    /// Advance the playfield simulation by one frame.
    pub fn advance(&mut self) {
        self.advance_ball();
    }

    fn draw_horizontal_line(&self, buffer: &mut [u8], x: usize, y: usize, length: usize) {
        let y_offset = y * DISPLAY_ROW_BYTES;
        for my_x in x..(x+length) {
            for i in 0..BYTES_PER_PIXEL {
                buffer[y_offset + my_x*BYTES_PER_PIXEL + i] = 0xFF;
            }
        }
    }

    fn draw_vertical_line(&self, buffer: &mut [u8], x: usize, y: usize, length: usize) {
        for my_y in y..(y+length) {
            let y_offset = my_y * DISPLAY_ROW_BYTES;
            for i in 0..BYTES_PER_PIXEL {
                buffer[y_offset + x*BYTES_PER_PIXEL + i] = 0xFF;
            }
        }
    }

    /// Draw the border around the playfield into the buffer.
    fn draw_playfield_border(&self, buffer: &mut [u8]) {
        debug_assert_eq!(buffer.len(), DISPLAY_BYTES);

        const BORDER_LEFT: usize = PLAYFIELD_LEFT - 1;
        const BORDER_HORIZONTAL_LENGTH: usize = PLAYFIELD_WIDTH.as_integer() as usize + 2;

        // top border
        self.draw_horizontal_line(
            buffer,
            BORDER_LEFT,
            PLAYFIELD_TOP - 1,
            BORDER_HORIZONTAL_LENGTH,
        );

        // bottom border
        self.draw_horizontal_line(
            buffer,
            BORDER_LEFT,
            PLAYFIELD_TOP + (PLAYFIELD_HEIGHT.as_integer() as usize) + 1,
            BORDER_HORIZONTAL_LENGTH,
        );
    }

    /// Draw the current state of the playfield onto the display.
    pub fn draw<DI: DisplayInterface>(&self, display_interface: &DI, peripherals: &mut Peripherals) {
        let mut screen = [0u8; DISPLAY_BYTES];

        // draw playfield border
        self.draw_playfield_border(&mut screen);

        /*
        // draw ball
        let ball_x = self.ball.position.x.as_integer() as usize;
        let ball_y = self.ball.position.y.as_integer() as usize;
        let ball_offset = ball_y * PLAYFIELD_ROW_ELEMENTS + ball_x * BYTES_PER_PIXEL;
        field[ball_offset+0] = 0xFF;
        field[ball_offset+1] = 0xFF;
        */

        DisplayCommand::WriteRam.transmit(display_interface, peripherals);
        display_interface.send(peripherals, None, &screen);
    }
}
