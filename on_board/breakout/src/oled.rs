//! Code for the 96x96 pixel PSP27801 OLED display controlled by the SSD1351 controller.


use atsamd21g::Peripherals;

use crate::iopin;
use crate::init::init_spi;
use crate::pin::Peripheral;


/// Low-level interface to the display.
pub trait DisplayInterface {
    fn set_up(&self, peripherals: &mut Peripherals);
    fn send(&self, peripherals: &mut Peripherals, command: Option<u8>, data: &[u8]);
    fn receive(&self, peripherals: &mut Peripherals, command: Option<u8>, buffer: &mut [u8]);
}


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressIncrement {
    Horizontal,
    Vertical,
}
impl AddressIncrement {
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::Horizontal => 0b0,
            Self::Vertical => 0b1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ColorDepth {
    Colors65k,
    Colors262k,
    Colors262k16BitFormat2,
}
impl ColorDepth {
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::Colors65k => 0b00,
            Self::Colors262k => 0b10,
            Self::Colors262k16BitFormat2 => 0b11,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParallelInterface {
    EightBit,
    SixteenBit,
    EighteenBit,
}
impl ParallelInterface {
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::EightBit => 0b00,
            Self::SixteenBit => 0b01,
            Self::EighteenBit => 0b11,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GpioState {
    HiZInputDisabled,
    HiZInputEnabled,
    OutputLow,
    OutputHigh,
}
impl GpioState {
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::HiZInputDisabled => 0b00,
            Self::HiZInputEnabled => 0b01,
            Self::OutputLow => 0b10,
            Self::OutputHigh => 0b11,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LockState {
    CommandsUnlocked,
    CommandsLocked,
    AdvancedCommandsLocked,
    AdvancedCommandsUnlocked,
}
impl LockState {
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::CommandsUnlocked => 0x12,
            Self::CommandsLocked => 0x16,
            Self::AdvancedCommandsLocked => 0xB0,
            Self::AdvancedCommandsUnlocked => 0x1B,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScrollSpeed {
    TestMode,
    Normal,
    Slow,
    Slowest,
}
impl ScrollSpeed {
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::TestMode => 0b00,
            Self::Normal => 0b01,
            Self::Slow => 0b10,
            Self::Slowest => 0b11,
        }
    }
}

/// A command that can be sent to the display.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DisplayCommand<'a> {
    SetColumnAddress { start: u8, end: u8 },
    SetRowAddress { start: u8, end: u8 },
    WriteRam,
    ReadRam,
    SetMapping {
        address_increment: AddressIncrement,
        reverse_columns: bool,
        swap_color_sequence: bool,
        backward_scan: bool,
        com_split_odd_even: bool,
        color_depth: ColorDepth,
    },
    SetDisplayStartLine { line: u8 },
    SetDisplayOffset { offset: u8 },
    DisplayModeAllOff,
    DisplayModeAllOn,
    DisplayModeRegular,
    DisplayModeInverted,
    FunctionSelection {
        internal_vdd_regulator: bool,
        interface: ParallelInterface,
    },
    NopAD,
    DisplayOff,
    DisplayOn,
    NopB0,
    SetPhasePeriods {
        phase1: u8,
        phase2: u8,
    },
    DisplayEnhancement { enhance: bool },
    SetFrequency {
        front_clock_divider_2pow: u8,
        oscillator_frequency: u8,
    },
    // SetSegmentLowVoltage: there is only one valid value; don't bother
    SetGpio {
        gpio0: GpioState,
        gpio1: GpioState,
    },
    SetSecondPreChargePeriod { period: u8 },
    SetGrayscaleLookUpTable { table: &'a [u8] },
    ResetGrayscaleLookUpTable,
    SetPreChargeVoltageLevel { voltage_level: u8 },
    SetComDeselectVoltageLevel { voltage_level: u8 },
    SetColorContrast {
        color1_contrast: u8,
        color2_contrast: u8,
        color3_contrast: u8,
    },
    SetMasterContrast { contrast: u8 },
    SetMuxRatio { ratio: u8 },
    NopD1,
    NopE3,
    SetCommandLock { state: LockState },
    HorizontalScroll {
        scrolling: u8,
        start_row: u8,
        scroll_row_count: u8,
        scroll_speed: ScrollSpeed,
    },
    StopMoving,
    StartMoving,
}
impl<'a> DisplayCommand<'a> {
    /// The code representing this command.
    #[inline]
    pub const fn as_command_code(&self) -> u8 {
        match self {
            Self::SetColumnAddress { .. } => 0x15,
            Self::SetRowAddress { .. } => 0x75,
            Self::WriteRam => 0x5C,
            Self::ReadRam => 0x5D,
            Self::SetMapping { .. } => 0xA0,
            Self::SetDisplayStartLine { .. } => 0xA1,
            Self::SetDisplayOffset { .. } => 0xA2,
            Self::DisplayModeAllOff => 0xA4,
            Self::DisplayModeAllOn => 0xA5,
            Self::DisplayModeRegular => 0xA6,
            Self::DisplayModeInverted => 0xA7,
            Self::FunctionSelection { .. } => 0xAB,
            Self::NopAD => 0xAD,
            Self::DisplayOff => 0xAE,
            Self::DisplayOn => 0xAF,
            Self::NopB0 => 0xB0,
            Self::SetPhasePeriods { .. } => 0xB1,
            Self::DisplayEnhancement { .. } => 0xB2,
            Self::SetFrequency { .. } => 0xB3,
            Self::SetGpio { .. } => 0xB5,
            Self::SetSecondPreChargePeriod { .. } => 0xB6,
            Self::SetGrayscaleLookUpTable { .. } => 0xB8,
            Self::ResetGrayscaleLookUpTable => 0xB9,
            Self::SetPreChargeVoltageLevel { .. } => 0xBB,
            Self::SetComDeselectVoltageLevel { .. } => 0xBE,
            Self::SetColorContrast { .. } => 0xC1,
            Self::SetMasterContrast { .. } => 0xC7,
            Self::SetMuxRatio { .. } => 0xCA,
            Self::NopD1 => 0xD1,
            Self::NopE3 => 0xE3,
            Self::SetCommandLock { .. } => 0xFD,
            Self::HorizontalScroll { .. } => 0x96,
            Self::StopMoving => 0x9E,
            Self::StartMoving => 0x9F,
        }
    }

    pub const fn is_valid(&self) -> bool {
        match self {
            Self::SetColumnAddress { start, end }
                => *start <= 127 && *end <= 127 && *start <= *end,
            Self::SetRowAddress { start, end }
            => *start <= 127 && *end <= 127 && *start <= *end,
            Self::WriteRam => true,
            Self::ReadRam => true,
            Self::SetMapping { .. } => true,
            Self::SetDisplayStartLine { line } => *line <= 127,
            Self::SetDisplayOffset { offset } => *offset <= 127,
            Self::DisplayModeAllOff => true,
            Self::DisplayModeAllOn => true,
            Self::DisplayModeRegular => true,
            Self::DisplayModeInverted => true,
            Self::FunctionSelection { .. } => true,
            Self::NopAD => true,
            Self::DisplayOff => true,
            Self::DisplayOn => true,
            Self::NopB0 => true,
            Self::SetPhasePeriods { phase1, phase2 }
                => *phase1 >= 2 && *phase1 <= 15 && *phase2 >= 3 && *phase2 <= 15,
            Self::DisplayEnhancement { .. } => true,
            Self::SetFrequency { front_clock_divider_2pow, oscillator_frequency }
                => *front_clock_divider_2pow <= 0b1010 && *oscillator_frequency <= 0b1111,
            Self::SetGpio { .. } => true,
            Self::SetSecondPreChargePeriod { period }
                => *period >= 0b0001 && *period <= 0b1111,
            Self::SetGrayscaleLookUpTable { table }
                => table.len() == 63,
            Self::ResetGrayscaleLookUpTable => true,
            Self::SetPreChargeVoltageLevel { voltage_level }
                => *voltage_level <= 0b1_1111,
            Self::SetComDeselectVoltageLevel { voltage_level }
                => *voltage_level <= 0b111,
            Self::SetColorContrast { .. } => true,
            Self::SetMasterContrast { contrast }
                => *contrast <= 0b1111,
            Self::SetMuxRatio { ratio }
                => *ratio >= 15 && *ratio <= 127,
            Self::NopD1 => true,
            Self::NopE3 => true,
            Self::SetCommandLock { .. } => true,
            Self::HorizontalScroll { start_row, scroll_row_count, .. }
                => *start_row <= 0b111_1111 && *start_row + *scroll_row_count <= 128,
            Self::StopMoving => true,
            Self::StartMoving => true,
        }
    }

    pub fn transmit<DI: DisplayInterface>(&self, display_interface: &DI, peripherals: &mut Peripherals) {
        debug_assert!(self.is_valid());

        match self {
            DisplayCommand::SetColumnAddress { start, end } => {
                display_interface.send(peripherals, Some(self.as_command_code()), &[*start, *end]);
            },
            DisplayCommand::SetRowAddress { start, end } => {
                display_interface.send(peripherals, Some(self.as_command_code()), &[*start, *end]);
            },
            DisplayCommand::SetMapping { address_increment, reverse_columns, swap_color_sequence, backward_scan, com_split_odd_even, color_depth } => {
                let data_byte =
                    (color_depth.as_bits() << 6)
                    | if *com_split_odd_even { 1 << 5 } else { 0 }
                    | if *backward_scan { 1 << 4 } else { 0 }
                    // 3 is reserved
                    | if *swap_color_sequence { 1 << 2 } else { 0 }
                    | if *reverse_columns { 1 << 1 } else { 0 }
                    | address_increment.as_bits() << 0
                ;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetDisplayStartLine { line } => {
                display_interface.send(peripherals, Some(self.as_command_code()), &[*line]);
            },
            DisplayCommand::SetDisplayOffset { offset } => {
                display_interface.send(peripherals, Some(self.as_command_code()), &[*offset]);
            },
            DisplayCommand::FunctionSelection { internal_vdd_regulator, interface } => {
                let data_byte =
                    (interface.as_bits() << 6)
                    | if *internal_vdd_regulator { 1 << 0 } else { 0 }
                ;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetPhasePeriods { phase1, phase2 } => {
                let data_byte =
                    *phase2 << 4
                    | *phase1 << 0
                ;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::DisplayEnhancement { enhance } => {
                let first_data_byte = if *enhance { 0xA4 } else { 0x00 };
                display_interface.send(peripherals, Some(self.as_command_code()), &[first_data_byte, 0x00, 0x00]);
            },
            DisplayCommand::SetFrequency { front_clock_divider_2pow, oscillator_frequency } => {
                let data_byte =
                    *oscillator_frequency << 4
                    | *front_clock_divider_2pow << 0
                ;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetGpio { gpio0, gpio1 } => {
                let data_byte =
                    gpio1.as_bits() << 2
                    | gpio0.as_bits() << 0
                ;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetSecondPreChargePeriod { period } => {
                let data_byte = *period;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetGrayscaleLookUpTable { table } => {
                display_interface.send(peripherals, Some(self.as_command_code()), *table);
            },
            DisplayCommand::SetPreChargeVoltageLevel { voltage_level } => {
                let data_byte = *voltage_level;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetComDeselectVoltageLevel { voltage_level } => {
                let data_byte = *voltage_level;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetColorContrast { color1_contrast, color2_contrast, color3_contrast } => {
                display_interface.send(peripherals, Some(self.as_command_code()), &[*color1_contrast, *color2_contrast, *color3_contrast]);
            },
            DisplayCommand::SetMasterContrast { contrast } => {
                let data_byte = *contrast;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetMuxRatio { ratio } => {
                let data_byte = *ratio;
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::SetCommandLock { state } => {
                let data_byte = state.as_bits();
                display_interface.send(peripherals, Some(self.as_command_code()), &[data_byte]);
            },
            DisplayCommand::HorizontalScroll { scrolling, start_row, scroll_row_count, scroll_speed } => {
                let data_bytes = [
                    *scrolling,
                    *start_row,
                    *scroll_row_count,
                    0x00,
                    scroll_speed.as_bits(),
                ];
                display_interface.send(peripherals, Some(self.as_command_code()), &data_bytes);
            },
            _ => {
                // command with no data
                display_interface.send(peripherals, Some(self.as_command_code()), &[]);
            },
        }
    }
}

pub struct ArduinoZeroClick1Interface;
impl ArduinoZeroClick1Interface {
    /// Transmits data to the display controller. You must pull down the ~CS pin before calling this
    /// function!
    fn internal_transmit(&self, peripherals: &mut Peripherals, is_command: bool, data: &[u8]) {
        // wait for SPI shift register to be ready for the next byte
        while peripherals.SERCOM1.spi().intflag.read().dre().bit_is_clear() {
        }

        if is_command {
            // pull data/~command pin down
            iopin!(set_low, peripherals, PA, 20);
        }

        // send
        for b in data {
            unsafe {
                peripherals.SERCOM1.spi().data.modify(|_, w| w
                    .data().bits(u16::from(*b))
                )
            };
            while peripherals.SERCOM1.spi().intflag.read().dre().bit_is_clear() {
            }
        }

        // wait for transmission to end fully
        while peripherals.SERCOM1.spi().intflag.read().txc().bit_is_clear() {
        }

        // always pull data/~command pin back up
        iopin!(set_high, peripherals, PA, 20);
    }
}
impl DisplayInterface for ArduinoZeroClick1Interface {
    fn set_up(&self, peripherals: &mut Peripherals) {
        // 1. set up pins for SPI
        // on SERCOM1: PA16 = COPI, PA17 = SCK, PA19 = CIPO
        iopin!(make_peripheral, peripherals, PA, 16, 17, 19);
        iopin!(select_peripheral, peripherals, Peripheral::C, PA, 16, 17, 19);
        // manually controlled: PA04 = ~RST, PA18 = ~CS, PA20 = D/~C, PA14 = EN
        iopin!(make_io, peripherals, PA, 4, 14, 18, 20);
        iopin!(make_output, peripherals, PA, 4, 14, 18, 20);
        iopin!(set_high, peripherals, PA, 18);
        iopin!(set_low, peripherals, PA, 4, 14, 20);

        crate::usart::write(peripherals, b"pin setup done\r\n");

        // 2. set up SPI
        init_spi(peripherals);
        crate::usart::write(peripherals, b"SPI setup done\r\n");

        // 3. power up display
        iopin!(set_high, peripherals, PA, 14);
        crate::usart::write(peripherals, b"display has been granted power\r\n");

        // 4. stop resetting display
        iopin!(set_high, peripherals, PA, 4);
        crate::usart::write(peripherals, b"display reset ended\r\n");

        // 5. clear out display RAM
        (DisplayCommand::SetColumnAddress { start: 0, end: 127 })
            .transmit(self, peripherals);
        (DisplayCommand::SetRowAddress { start: 0, end: 127 })
            .transmit(self, peripherals);
        DisplayCommand::WriteRam.transmit(self, peripherals);
        for _ in 0..(128*128)/32 {
            let chunk = [0u8; 32*2];
            self.send(peripherals, None, &chunk);
        }

        // 6. start at row and column as actually connected to the display
        (DisplayCommand::SetColumnAddress { start: 16, end: 111 })
            .transmit(self, peripherals);
        (DisplayCommand::SetRowAddress { start: 0, end: 95 })
            .transmit(self, peripherals);

        // 7. stop sleeping
        DisplayCommand::DisplayOn.transmit(self, peripherals);
        crate::usart::write(peripherals, b"sent display-on command\r\n");
    }

    fn send(&self, peripherals: &mut Peripherals, command: Option<u8>, data: &[u8]) {
        // wait for SPI shift register to be ready for the next byte
        while peripherals.SERCOM1.spi().intflag.read().dre().bit_is_clear() {
        }

        // pin-select the display controller
        iopin!(set_low, peripherals, PA, 18);

        if let Some(cmd) = command {
            self.internal_transmit(peripherals, true, &[cmd]);
        }
        self.internal_transmit(peripherals, false, data);

        // unselect the display controller
        iopin!(set_high, peripherals, PA, 18);
    }

    fn receive(&self, peripherals: &mut Peripherals, command: Option<u8>, buffer: &mut [u8]) {
        // wait for SPI shift register to be ready for the next byte
        while peripherals.SERCOM1.spi().intflag.read().dre().bit_is_clear() {
        }

        // pin-select the display controller
        iopin!(set_low, peripherals, PA, 18);

        if let Some(cmd) = command {
            self.internal_transmit(peripherals, true, &[cmd]);
        }

        for b in buffer {
            // transmit zero byte
            unsafe {
                peripherals.SERCOM1.spi().data.modify(|_, w| w
                    .data().bits(0)
                )
            };
            while peripherals.SERCOM1.spi().intflag.read().dre().bit_is_clear() {
            }

            // wait until byte has been received
            while peripherals.SERCOM1.spi().intflag.read().rxc().bit_is_clear() {
            }

            // read it out
            *b = (peripherals.SERCOM1.spi().data.read().data().bits() & 0xFF) as u8;
        }

        // wait for transmission to end fully
        while peripherals.SERCOM1.spi().intflag.read().txc().bit_is_clear() {
        }

        // unselect the display controller
        iopin!(set_high, peripherals, PA, 18);
    }
}
