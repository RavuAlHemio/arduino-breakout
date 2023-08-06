#![no_main]
#![no_std]


mod calib;
mod init;
mod keypad;
mod oled;
mod pin;
mod playfield;
mod spi;
mod timer;
mod usart;


use core::panic::PanicInfo;

use breakout_common::fixedpoint::FixedPoint;
use atsamd21g::Peripherals;
use cortex_m::Peripherals as CorePeripherals;
use cortex_m_rt::{entry, exception};

use crate::oled::{ArduinoZeroClick1Interface, DisplayInterface};
use crate::playfield::Playfield;


#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    // UNSAFE: we can steal the peripherals here because no other code is being executed
    // and we are not returning to the code that produced this panic
    let mut peripherals = unsafe { Peripherals::steal() };

    crate::usart::write(&mut peripherals, b"OH NO\r\n");
    if let Some(loc) = info.location() {
        crate::usart::write(&mut peripherals, b"blew up in file ");
        crate::usart::write(&mut peripherals, loc.file().as_bytes());
        crate::usart::write(&mut peripherals, b" on line 0x");

        for nibble_index in 0..8 {
            let shift_count = (7 - nibble_index) * 4;
            let nibble = ((loc.line() >> shift_count) & 0xF) as u8;
            let b = nibble_to_hex_byte(nibble);
            crate::usart::write(&mut peripherals, &[b]);
        }

        crate::usart::write(&mut peripherals, b"\r\n");
    }

    loop {
    }
}



#[exception]
fn SysTick() {
    crate::timer::tick();
}


fn nibble_to_hex_byte(nibble: u8) -> u8 {
    match nibble {
        0x0..=0x9 => nibble + b'0',
        0xA..=0xF => nibble - 10 + b'A',
        _ => b'?',
    }
}


fn i16_to_hex_bytes(mut val: i16, buf: &mut [u8]) -> &[u8] {
    if val == i16::MIN {
        // we cannot negate this value
        buf[0] = b'-';
        buf[1] = b'0';
        buf[2] = b'x';
        buf[3] = b'8';
        buf[4] = b'0';
        buf[5] = b'0';
        buf[6] = b'0';
        return &buf[0..7];
    }
    if val == 0 {
        // theoretically zero digits...
        buf[0] = b'0';
        buf[1] = b'x';
        buf[2] = b'0';
        return &buf[0..3];
    }

    let minus = if val < 0 {
        val = -val;
        true
    } else {
        false
    };

    let mut i: usize = 0;
    while val > 0 {
        let nibble = (val & 0xF) as u8;
        buf[i] = nibble_to_hex_byte(nibble);
        i += 1;
        val >>= 4;
    }

    buf[i] = b'x';
    i += 1;
    buf[i] = b'0';
    i += 1;

    if minus {
        buf[i] = b'-';
        i += 1;
    }

    buf[0..i].reverse();

    &buf[0..i]
}


#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take()
        .expect("peripherals already taken?!");
    let mut core_peripherals = CorePeripherals::take()
        .expect("core peripherals already taken?!");

    // give power to PORT
    peripherals.PM.apbbmask.modify(|_, w| w
        .port_().set_bit()
    );

    // set up clock and timer
    crate::init::init_clock(&mut peripherals);
    crate::timer::set_up(&mut core_peripherals);

    // set up EDBG UART
    crate::init::init_edbg_uart(&mut peripherals);

    // set up SPI and display
    let display = ArduinoZeroClick1Interface;
    display.set_up(&mut peripherals);

    // set up keypad
    crate::keypad::setup_keypad_pins(&mut peripherals);

    // set up the playfield
    let mut playfield = Playfield::new();

    // move the ball a bit along the X axis for more interesting patterns
    playfield.ball.position.x += FixedPoint::new_integer(7);

    let mut delay_counter: u8 = 0;
    loop {
        // read keypad state
        let state = crate::keypad::read_keypad(&mut peripherals);
        // TODO: process keypad state

        delay_counter += 1;
        if delay_counter == 2 {
            delay_counter = 0;
            playfield.advance();
        }

        playfield.draw(&display, &mut peripherals);

        /*
        let mut ball_x_hex_buf = [0u8; 7];
        let mut ball_y_hex_buf = [0u8; 7];

        let ball_x_hex = i16_to_hex_bytes(playfield.ball.position.x.as_raw(), &mut ball_x_hex_buf);
        let ball_y_hex = i16_to_hex_bytes(playfield.ball.position.y.as_raw(), &mut ball_y_hex_buf);

        crate::usart::write(&mut peripherals, b"ball is at (");
        crate::usart::write(&mut peripherals, ball_x_hex);
        crate::usart::write(&mut peripherals, b", ");
        crate::usart::write(&mut peripherals, ball_y_hex);
        crate::usart::write(&mut peripherals, b")\r\n");

        let ball_x_hex = i16_to_hex_bytes(playfield.ball.velocity.x.as_raw(), &mut ball_x_hex_buf);
        let ball_y_hex = i16_to_hex_bytes(playfield.ball.velocity.y.as_raw(), &mut ball_y_hex_buf);

        crate::usart::write(&mut peripherals, b"ball is flying (");
        crate::usart::write(&mut peripherals, ball_x_hex);
        crate::usart::write(&mut peripherals, b", ");
        crate::usart::write(&mut peripherals, ball_y_hex);
        crate::usart::write(&mut peripherals, b")\r\n");
        */

        // TODO: delay?
    }
}
