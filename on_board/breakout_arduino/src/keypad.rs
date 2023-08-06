//! Code for the SPI keypad. It is assumed that it is on mikroBUS slot 2.


use atsamd21g::Peripherals;

use crate::iopin;
use crate::spi::{Sercom1Spi, Spi};


pub struct KeypadState {
    state: u16,
}
macro_rules! impl_is_pressed {
    ($name:ident, $mask:expr) => {
        pub const fn $name(&self) -> bool {
            (self.state & $mask) == 0
        }
    };
}
impl KeypadState {
    impl_is_pressed!(is_1_pressed, 0b0000_0000_0000_0001);
    impl_is_pressed!(is_2_pressed, 0b0000_0000_0000_0010);
    impl_is_pressed!(is_3_pressed, 0b0000_0000_0000_0100);
    impl_is_pressed!(is_a_pressed, 0b0000_0000_0000_1000);
    impl_is_pressed!(is_4_pressed, 0b0000_0000_0001_0000);
    impl_is_pressed!(is_5_pressed, 0b0000_0000_0010_0000);
    impl_is_pressed!(is_6_pressed, 0b0000_0000_0100_0000);
    impl_is_pressed!(is_b_pressed, 0b0000_0000_1000_0000);
    impl_is_pressed!(is_7_pressed, 0b0000_0001_0000_0000);
    impl_is_pressed!(is_8_pressed, 0b0000_0010_0000_0000);
    impl_is_pressed!(is_9_pressed, 0b0000_0100_0000_0000);
    impl_is_pressed!(is_c_pressed, 0b0000_1000_0000_0000);
    // bottom row is different
    impl_is_pressed!(is_0_pressed, 0b0001_0000_0000_0000);
    impl_is_pressed!(is_hash_pressed, 0b0010_0000_0000_0000);
    impl_is_pressed!(is_d_pressed, 0b0100_0000_0000_0000);
    impl_is_pressed!(is_asterisk_pressed, 0b1000_0000_0000_0000);

    pub fn output_to_uart(&self, peripherals: &mut Peripherals) {
        let mut buf = [0u8; 16];
        let mut i = 0;

        macro_rules! append_pressed {
            ($test_func:ident, $byte:expr) => {
                if self.$test_func() {
                    buf[i] = $byte;
                    i += 1;
                }
            };
        }

        append_pressed!(is_0_pressed, b'0');
        append_pressed!(is_1_pressed, b'1');
        append_pressed!(is_2_pressed, b'2');
        append_pressed!(is_3_pressed, b'3');
        append_pressed!(is_4_pressed, b'4');
        append_pressed!(is_5_pressed, b'5');
        append_pressed!(is_6_pressed, b'6');
        append_pressed!(is_7_pressed, b'7');
        append_pressed!(is_8_pressed, b'8');
        append_pressed!(is_9_pressed, b'9');
        append_pressed!(is_asterisk_pressed, b'*');
        append_pressed!(is_hash_pressed, b'#');
        append_pressed!(is_a_pressed, b'A');
        append_pressed!(is_b_pressed, b'B');
        append_pressed!(is_c_pressed, b'C');
        append_pressed!(is_d_pressed, b'D');

        crate::usart::write(peripherals, &buf[0..i]);
    }
}

/// Setup the keypad-specific pins. This assumes that SPI is already initialized.
pub fn setup_keypad_pins(peripherals: &mut Peripherals) {
    // ~RST = PB9, CS = PA7 (non-negated!)
    iopin!(make_io, peripherals, PA, 7);
    iopin!(make_io, peripherals, PB, 9);
    iopin!(make_output, peripherals, PA, 7);
    iopin!(make_output, peripherals, PB, 9);
    iopin!(set_low, peripherals, PA, 7);
    iopin!(set_high, peripherals, PB, 9);
}

pub fn read_keypad(peripherals: &mut Peripherals) -> KeypadState {
    // prepare everything
    let spi = Sercom1Spi;
    let mut buf = [0u8; 2];

    // pull chip select high (it's non-negated here!)
    iopin!(set_high, peripherals, PA, 7);

    // read 16 bits
    spi.exchange_data(peripherals, &mut buf);

    // pull chip select low again
    iopin!(set_low, peripherals, PA, 7);

    let state =
        (u16::from(buf[0]) << 8)
        | (u16::from(buf[1]) << 0)
    ;
    KeypadState {
        state
    }
}
