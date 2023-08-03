use arduino_breakout_common::fixedpoint::FixedPoint;
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


const PLAYFIELD_WIDTH: FixedPoint = FixedPoint::new_integer(96);
const PLAYFIELD_HEIGHT: FixedPoint = FixedPoint::new_integer(96);

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

    /// Draw the current state of the playfield onto the display.
    pub fn draw<DI: DisplayInterface>(&self, display_interface: &DI, peripherals: &mut Peripherals) {
        const BYTES_PER_PIXEL: usize = 2; // R5:G6:B5 encoding
        const PLAYFIELD_ROW_ELEMENTS: usize =
            PLAYFIELD_WIDTH.as_integer() as usize
            * BYTES_PER_PIXEL
        ;
        const PLAYFIELD_ELEMENTS: usize =
            PLAYFIELD_HEIGHT.as_integer() as usize
            * PLAYFIELD_ROW_ELEMENTS
        ;
        let mut field = [0u8; PLAYFIELD_ELEMENTS];

        // draw ball
        let ball_x = self.ball.position.x.as_integer() as usize;
        let ball_y = self.ball.position.y.as_integer() as usize;
        let ball_offset = ball_y * PLAYFIELD_ROW_ELEMENTS + ball_x * BYTES_PER_PIXEL;
        field[ball_offset+0] = 0xFF;
        field[ball_offset+1] = 0xFF;

        DisplayCommand::WriteRam.transmit(display_interface, peripherals);
        display_interface.send(peripherals, None, &field);
    }
}
