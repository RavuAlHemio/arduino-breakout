#![no_main]
#![no_std]


mod calib;
mod init;
mod keypad;
mod oled;
mod pin;
mod spi;
mod usart;


use core::panic::PanicInfo;

use atsamd21g::Peripherals;
use cortex_m_rt::entry;

use crate::oled::{ArduinoZeroClick1Interface, DisplayCommand, DisplayInterface};


#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    // UNSAFE: we can steal the peripherals here because no other code is being executed
    // and we are not returning to the code that produced this panic
    let mut peripherals = unsafe { Peripherals::steal() };

    loop {
    }
}


#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take()
        .expect("peripherals already taken?!");

    // give power to PORT
    peripherals.PM.apbbmask.modify(|_, w| w
        .port_().set_bit()
    );

    // set up clock
    crate::init::init_clock(&mut peripherals);

    // set up EDBG UART
    crate::init::init_edbg_uart(&mut peripherals);

    // set up SPI and display
    let display = ArduinoZeroClick1Interface;
    display.set_up(&mut peripherals);

    // set up keypad
    crate::keypad::setup_keypad_pins(&mut peripherals);

    // show image on display
    DisplayCommand::WriteRam.transmit(&display, &mut peripherals);
    let blahaj = include_bytes!("../../../blahaj.bin");
    display.send(&mut peripherals, None, blahaj);

    loop {
        // read keypad state
        let state = crate::keypad::read_keypad(&mut peripherals);
        crate::usart::write(&mut peripherals, b"keypad button state: ");
        state.output_to_uart(&mut peripherals);
        crate::usart::write(&mut peripherals, b"\r\n");

        for _ in 0..(2*1024*1024) {
            cortex_m::asm::nop();
        }
    }
}
