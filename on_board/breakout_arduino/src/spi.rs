//! Interfacing with the SAM D21's Serial Peripheral Interface capabilities.

use atsamd21g::Peripherals;


pub trait Spi {
    fn get_sercom_spi<'a>(&self, peripherals: &'a mut Peripherals) -> &'a atsamd21g::sercom0::SPI;

    fn wait_for_ready(&self, peripherals: &mut Peripherals) {
        let sercom_spi = self.get_sercom_spi(peripherals);

        // wait for SPI shift register to be ready for the next byte
        while sercom_spi.intflag.read().dre().bit_is_clear() {
        }
    }

    fn send_data(&self, peripherals: &mut Peripherals, data: &[u8]) {
        let sercom_spi = self.get_sercom_spi(peripherals);

        // wait for SPI shift register to be ready for the next byte
        while sercom_spi.intflag.read().dre().bit_is_clear() {
        }

        // send
        for b in data {
            unsafe {
                sercom_spi.data.modify(|_, w| w
                    .data().bits(u16::from(*b))
                )
            };
            while sercom_spi.intflag.read().dre().bit_is_clear() {
            }
        }

        // wait for transmission to end fully
        while sercom_spi.intflag.read().txc().bit_is_clear() {
        }
    }

    fn exchange_data(&self, peripherals: &mut Peripherals, data: &mut [u8]) {
        let sercom_spi = self.get_sercom_spi(peripherals);

        // wait for SPI shift register to be ready for the next byte
        while sercom_spi.intflag.read().dre().bit_is_clear() {
        }

        // send and receive
        for b in data {
            unsafe {
                sercom_spi.data.modify(|_, w| w
                    .data().bits(u16::from(*b))
                )
            };
            while sercom_spi.intflag.read().dre().bit_is_clear() {
            }

            // wait until byte has been received
            while sercom_spi.intflag.read().rxc().bit_is_clear() {
            }

            // read it out
            *b = (sercom_spi.data.read().data().bits() & 0xFF) as u8;
        }

        // wait for transmission to end fully
        while sercom_spi.intflag.read().txc().bit_is_clear() {
        }
    }
}

pub struct Sercom1Spi;
impl Spi for Sercom1Spi {
    fn get_sercom_spi<'a>(&self, peripherals: &'a mut Peripherals) -> &'a atsamd21g::sercom0::SPI {
        peripherals.SERCOM1.spi()
    }
}
