//! Functions for reading calibration values burned into the microcontroller during manufacturing.
//!
//! The calibration area is a 128-bit block at memory location 0x00806020, split up as such:
//!
//! ```plain
//! +-----------------------------------------------+
//! | 127 | 126 | 125 | 124 | 123 | 122 | 121 | 120 |
//! | reserved                                      |
//! ...
//! |  71 |  70 |  69 |  68 |  67 |  66 |  65 |  64 |
//! | reserved                                      |
//! +-----------------------------------------------+
//! |  63 |  62 |  61 |  60 |  59 |  58 |  57 |  56 |
//! | DFLL48M COARSE CAL                | USB TRIM  >
//! +-----------------------------------------------+
//! |  55 |  54 |  53 |  52 |  51 |  50 |  49 |  48 |
//! > USB | USB TRANSP                  | USB       >
//! > TRIM|                             | TRANSN    >
//! +-----------------------------------------------+
//! |  47 |  46 |  45 |  44 |  43 |  42 |  41 |  40 |
//! > USB TRANSN      | OSC32K CAL                  >
//! +-----------------------------------------------+
//! |  39 |  38 |  37 |  36 |  35 |  34 |  33 |  32 |
//! > OSC32K CAL| ADC BIASCAL     | ADC LINEARITY   >
//! +-----------------------------------------------+
//! |  31 |  30 |  29 |  28 |  27 |  26 |  25 |  24 |
//! > ADC LINEARITY               | reserved        |
//! +-----------------------------------------------+
//! |  23 |  22 |  21 |  20 |  19 |  18 |  17 |  16 |
//! | reserved                                      |
//! ...
//! |   7 |   6 |   5 |   4 |   3 |   2 |   1 |   0 |
//! | reserved                                      |
//! +-----------------------------------------------+
//! ```
//!
//! Since all non-reserved calibration values are in the bottom half and the SAM line is a
//! little-endian ARM variant, it is sufficient to read it as a 64-bit value.


use core::ptr::read_volatile;


const CALIBRATION_AREA: *const u64 = 0x0080_6020 as *const u64;


/// Obtain the ADC linearity calibration value. Store in `ADC.calib.linearity_cal`.
#[allow(unused)]
pub fn adc_linearity() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 27) & 0b1111_1111) as u8
}

/// Obtain the ADC bias calibration value. Store in `ADC.calib.bias_cal`.
#[allow(unused)]
pub fn adc_bias() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 35) & 0b111) as u8
}

/// Obtain the OSC32K calibration value. Store in `SYSCTRL.osc32k.calib`.
#[allow(unused)]
pub fn osc32k() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 38) & 0b111_1111) as u8
}

/// Obtain the USB TRANSN calibration value. Store in `USB.padcal.transn`.
#[allow(unused)]
pub fn usb_transn() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 45) & 0b1_1111) as u8
}

/// Obtain the USB TRANSP calibration value. Store in `USB.padcal.transp`.
#[allow(unused)]
pub fn usb_transp() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 50) & 0b1_1111) as u8
}

/// Obtain the USB TRIM calibration value. Store in `USB.padcal.trim`.
#[allow(unused)]
pub fn usb_trim() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 55) & 0b111) as u8
}

/// Obtain the DFLL48M coarse calibration value. Store in `SYSCTRL.dfllval.coarse` during setup of
/// DFLL48M in closed-loop mode (see datasheet § 17.6.7.1.2).
#[allow(unused)]
pub fn dfll48m_coarse() -> u8 {
    ((unsafe { read_volatile(CALIBRATION_AREA) } >> 58) & 0b11_1111) as u8
}
