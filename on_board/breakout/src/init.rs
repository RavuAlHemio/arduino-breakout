//! Initialization code.


use atsamd21g::Peripherals;

use crate::iopin;
use crate::pin::Peripheral;


/// Initialize clocks.
///
/// The SAM D21 starts with an 1 MHz clock derived from its internal 8 MHz oscillator. It can run at
/// speeds of up to 48 MHz; since the external oscillators only have lower maximum speeds (XOSC goes
/// up to 32 MHz, XOSC32K only allows exactly 32768 Hz), the method to obtain 48 MHz is to feed the
/// digital frequency locked loop (DFLL48M) with an appropriate reference (max. 33 kHz => use
/// XOSC32K) and then use its output.
///
/// The Arduino Zero board has a 32768 Hz oscillator connected to the D21's XOSC32K pins; its other
/// oscillator at 12 MHz is connected to the AVR32 microcontroller (used as an in-circuit
/// programmer) and therefore not usable here.
///
/// We are therefore looking at the following setup:
/// ```plain
///   ┌──────────┐     ┌──────────┐    ┌────────┐
///   │ XOSC32K  ├─┐ ┌─┤ GCG0     ├────┤ CPU    │
///   │ 32768 Hz │ │ │ │ 48 MHz   │    │ 48 MHz │
///   └──────────┘ │ │ └──────────┘    └────────┘
///                 ╳
///   ┌──────────┐ │ │ ┌──────────┐
/// ┌─┤ DFLL48M  ├─┘ └─┤ GCG1     ├─┐
/// │ │ 48 MHz   │     │ 32768 Hz │ │
/// │ └──────────┘     └──────────┘ │
/// │                               │
/// │      (GCLK_DFLL48M_REF)       │
/// └───────────────────────────────┘
/// ```
///
/// (The CPU is always connected to GCG0 while other clocks such as the reference clock for the
/// DFLL48M can be linked as needed.)
pub(crate) fn init_clock(peripherals: &mut Peripherals) {
    // set flash wait state to match 48 MHz
    peripherals.NVMCTRL.ctrlb.modify(|_, w| w
        .rws().half()
    );

    // give power to SYSCTRL and GCLK
    peripherals.PM.apbamask.modify(|_, w| w
        .sysctrl_().set_bit()
        .gclk_().set_bit()
    );

    // configure XOSC32K
    unsafe {
        peripherals.SYSCTRL.xosc32k.modify(|_, w| w
            .xtalen().set_bit() // a crystal is connected, not a clock
            .en32k().set_bit() // enable 32 kHz output
            .aampen().set_bit() // enable automatic amplitude control
            .runstdby().set_bit() // run in standby
            .ondemand().clear_bit() // always run, even if no peripheral requests us
            .startup().bits(0x7) // longest startup time to ensure stability
            .wrtlock().clear_bit()
        )
    };

    // enable XOSC32K (must be a separate call)
    peripherals.SYSCTRL.xosc32k.modify(|_, w| w
        .enable().set_bit()
    );
    while peripherals.SYSCTRL.pclksr.read().xosc32krdy().bit_is_clear() {
    }

    // reset GCLK
    peripherals.GCLK.ctrl.modify(|_, w| w
        .swrst().set_bit()
    );
    while peripherals.GCLK.ctrl.read().swrst().bit_is_set() || peripherals.GCLK.status.read().syncbusy().bit_is_set() {
    }

    // set up GCG1 with XOSC32K (undivided) as source
    unsafe {
        peripherals.GCLK.gendiv.modify(|_, w| w
            .id().bits(1)
            .div().bits(1)
        )
    };
    while peripherals.GCLK.status.read().syncbusy().bit_is_set() {
    }
    unsafe {
        peripherals.GCLK.genctrl.modify(|_, w| w
            .id().bits(1)
            .src().xosc32k()
            .idc().clear_bit() // don't improve duty cycle (doesn't make sense with divisor 1)
            .oov().clear_bit() // pin output is zero if clock is disabled (doesn't actually matter)
            .oe().clear_bit() // don't output on a pin
            .divsel().clear_bit() // divide by gendiv.div, not by 2**(gendiv.div + 1)
            .runstdby().set_bit() // keep running even in standby
            .genen().set_bit() // enable it
        )
    };
    while peripherals.GCLK.status.read().syncbusy().bit_is_set() {
    }

    // plug GCG1 into DFLL48M reference
    peripherals.GCLK.clkctrl.modify(|_, w| w
        .id().dfll48()
        .gen().gclk1()
        .clken().set_bit()
    );

    // ensure DFLL48M is ready
    while peripherals.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {
    }

    // force DFLL48M to always be available (silicon erratum)
    peripherals.SYSCTRL.dfllctrl.modify(|_, w| w
        .ondemand().clear_bit() // always run the clock
    );
    while peripherals.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {
    }

    // configure DFLL48M multiplier
    unsafe {
        peripherals.SYSCTRL.dfllmul.modify(|_, w| w
            .cstep().bits(0b11_1111 / 2)
            .fstep().bits(0b11_1111_1111 / 2)
            .mul().bits((48_000_000 / 32_768) as u16)
        )
    };
    while peripherals.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {
    }

    // preload coarse calibration value
    unsafe {
        peripherals.SYSCTRL.dfllval.modify(|_, w| w
            .coarse().bits(crate::calib::dfll48m_coarse())
        )
    };
    while peripherals.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {
    }

    // configure DFLL48M
    peripherals.SYSCTRL.dfllctrl.modify(|_, w| w
        .mode().set_bit() // closed-loop operation
        .usbcrm().clear_bit() // disable USB clock recovery mode
        .ondemand().clear_bit() // always run the clock
        .qldis().set_bit() // disable quick lock
        .bplckc().set_bit() // bypass coarse lock (we have preloaded the calibration value)
        .waitlock().set_bit() // wait until lock
    );
    while peripherals.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {
    }

    // start DFLL48M
    peripherals.SYSCTRL.dfllctrl.modify(|_, w| w
        .enable().set_bit()
    );
    while peripherals.SYSCTRL.pclksr.read().dfllrdy().bit_is_clear() {
    }

    // wait for DFLL48M to stabilize
    while peripherals.SYSCTRL.pclksr.read().dflllckc().bit_is_clear() || peripherals.SYSCTRL.pclksr.read().dflllckf().bit_is_clear() {
    }

    // set up GCG0 (main clock) with DFLL48M (undivided) as source
    unsafe {
        peripherals.GCLK.gendiv.modify(|_, w| w
            .id().bits(0)
            .div().bits(1)
        )
    };
    while peripherals.GCLK.status.read().syncbusy().bit_is_set() {
    }
    unsafe {
        peripherals.GCLK.genctrl.modify(|_, w| w
            .id().bits(0)
            .src().dfll48m()
            .idc().clear_bit() // don't improve duty cycle (doesn't make sense with divisor 1)
            .oov().clear_bit() // pin output is zero if clock is disabled (doesn't actually matter)
            .oe().clear_bit() // don't output on a pin
            .divsel().clear_bit() // divide by gendiv.div, not by 2**(gendiv.div + 1)
            .runstdby().set_bit() // keep running even in standby
            .genen().set_bit() // enable it
        )
    };
    while peripherals.GCLK.status.read().syncbusy().bit_is_set() {
    }
}


/// Initializes SPI on SERCOM1, matching the canonical Arduino Zero pinout.
pub(crate) fn init_spi(peripherals: &mut Peripherals) {
    // pins:
    // PA16 (COPI) to SERCOM1 PAD[0] (peripheral C)
    // PA17 (SCK)  to SERCOM1 PAD[1] (peripheral C)
    // PA19 (CIPO) to SERCOM1 PAD[3] (peripheral C)
    iopin!(make_peripheral, peripherals, PA, 16, 17, 19);
    iopin!(select_peripheral, peripherals, Peripheral::C, PA, 16, 17, 19);

    // give power to SERCOM1
    peripherals.PM.apbcmask.modify(|_, w| w
        .sercom1_().set_bit()
    );

    // connect GCLK0 (main CPU clock) to SERCOM1
    peripherals.GCLK.clkctrl.modify(|_, w| w
        .id().sercom1_core()
        .gen().gclk0()
        .clken().set_bit()
    );

    // reset SERCOM1
    let sercom1_spi = peripherals.SERCOM1.spi();
    sercom1_spi.ctrla.modify(|_, w| w
        .swrst().set_bit()
    );
    while sercom1_spi.ctrla.read().swrst().bit_is_set() && sercom1_spi.syncbusy.read().swrst().bit_is_set() {
    }

    // switch SERCOM1 to SPI controller mode
    sercom1_spi.ctrla.modify(|_, w| w
        .mode().spi_master()
    );
    // (no synchronization)

    // turn on debug LED
    //iopin!(set_high, peripherals, PA, 17);

    // display controller says:
    // * clock idle high (CPOL=1)
    // * data sampled at second edge (CHPA=1)
    // * MSB first (DORD=0)
    unsafe {
        sercom1_spi.ctrla.modify(|_, w| w
            .dopo().bits(0) // COPI on pad 0, SCK on pad 1, ~CS theoretically on pad 2 (unused)
            .dipo().bits(3) // CIPO on pad 3
            .form().bits(0) // data format: SPI frame without address
            .cpha().set_bit() // data sampled at trailing edge
            .cpol().set_bit() // clock idle high
            .dord().clear_bit() // MSB first
        )
    };
    // (no synchronization)

    unsafe {
        sercom1_spi.ctrlb.modify(|_, w| w
            .chsize().bits(0) // 8 bits per byte
            .ssde().clear_bit() // no wakeup on ~CS fall
            .mssen().clear_bit() // no control of ~CS pin through SERCOM (we do it manually)
            .rxen().set_bit() // enable receiver
        )
    };
    while sercom1_spi.syncbusy.read().ctrlb().bit_is_set() {
    }

    // set baud rate
    // display controller says: clock cycle: min. 50 ns/b => baud rate: max. 20 Mb/s
    // SAM D21 says: clock cycle: typ. 84 ns/b => baud rate: typ. 11.9 Mb/s
    // let's say 10 Mb for simplicity
    // clock generation is in synchronous mode because this isn't UART
    // BAUD = f_{ref} / (2 * f_{BAUD}) - 1
    //      = 48_000_000 / (2 * 10_000_000) - 1
    //      = 48_000_000 / 20_000_000 - 1
    //      = 2.4 - 1
    //      = 1.4
    //      ~ 2 [round to slower; be conservative]
    unsafe {
        sercom1_spi.baud.modify(|_, w| w
            .baud().bits(4)
        )
    };
    // (no synchronization)

    // turn on SPI
    sercom1_spi.ctrla.modify(|_, w| w
        .enable().set_bit()
    );
    while sercom1_spi.syncbusy.read().enable().bit_is_set() {
    }
}


/// Initializes UART on SERCOM5, communicating with the EDBG virtual COM port.
pub fn init_edbg_uart(peripherals: &mut Peripherals) {
    // pins:
    // PB22 (TXD) to SERCOM5 PAD[2] (peripheral D)
    // PB23 (RXD) to SERCOM5 PAD[3] (peripheral D)
    iopin!(make_peripheral, peripherals, PB, 22, 23);
    iopin!(select_peripheral, peripherals, Peripheral::D, PB, 22, 23);

    // give power to SERCOM5
    peripherals.PM.apbcmask.modify(|_, w| w
        .sercom5_().set_bit()
    );

    // connect GCLK0 (main CPU clock) to SERCOM5
    peripherals.GCLK.clkctrl.modify(|_, w| w
        .id().sercom5_core()
        .gen().gclk0()
        .clken().set_bit()
    );

    let sercom5_usart = peripherals.SERCOM5.usart();

    // reset SERCOM5
    sercom5_usart.ctrla.modify(|_, w| w
        .swrst().set_bit()
    );
    while sercom5_usart.ctrla.read().swrst().bit_is_set() && sercom5_usart.syncbusy.read().swrst().bit_is_set() {
    }

    // switch SERCOM5 to USART mode
    sercom5_usart.ctrla.modify(|_, w| w
        .mode().usart_int_clk()
    );
    // (no synchronization)

    // set up SERCOM5
    unsafe {
        sercom5_usart.ctrla.modify(|_, w| w
            .sampr().bits(0x0) // 16x oversampling, arithmetic baud rate generation
            .txpo().bits(0x1) // transmit on PAD[2], external clock (unused) on PAD[3], no RTS/CTS
            .rxpo().bits(0x3) // receive on PAD[3]
            .sampa().bits(0x0) // use samples 7-8-9
            .form().bits(0x0) // regular USART frame, no parity
            .cmode().clear_bit() // async communication (UART, not USRT)
            .dord().set_bit() // LSB first (RS-232 standard)
        )
    };
    // (no synchronization)

    unsafe {
        sercom5_usart.ctrlb.modify(|_, w| w
            .chsize().bits(0x0) // 8 bits per byte
            .sbmode().clear_bit() // single stop bit
            .colden().clear_bit() // no collision detection
            .sfde().clear_bit() // disable start-of-frame detection
            .enc().clear_bit() // regular (non-IrDA) encoding
            .txen().set_bit() // enable transmitter
            .rxen().set_bit() // enable receiver
        )
    };
    // (no synchronization -- txen/rxen are only synchronized if the USART is enabled)

    // set to 115_200 baud (arithmetic baud rate generation as chosen above)
    // BAUD = 65_536 * (1 - S * (f_{BAUD} / f_{ref}))
    //      = 65_536 * (1 - 16 * (115_200 / 48_000_000))
    //      = 65_536 * (1 - 16 * 0.0024)
    //      = 65_536 * (1 - 0.0384)
    //      = 65_536 * 0.9616
    //      = 63_019.4176
    //      ~ 63_019
    unsafe {
        sercom5_usart.baud().modify(|_, w| w
            .baud().bits(63_019)
        )
    };
    // (no synchronization)

    // enable USART
    sercom5_usart.ctrla.modify(|_, w| w
        .enable().set_bit()
    );
    while sercom5_usart.syncbusy.read().enable().bit_is_set() {
    }
}
