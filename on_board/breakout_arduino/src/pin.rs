//! Macros to make manipulating digital I/O pins more straightforward.


/// One of the peripherals to choose in the peripheral multiplexer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(unused)]
pub enum Peripheral {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}
impl Peripheral {
    #[inline]
    #[allow(unused)]
    pub const fn as_bits(&self) -> u8 {
        match self {
            Self::A => 0x0,
            Self::B => 0x1,
            Self::C => 0x2,
            Self::D => 0x3,
            Self::E => 0x4,
            Self::F => 0x5,
            Self::G => 0x6,
            Self::H => 0x7,
        }
    }
}
macro_rules! impl_from_peripheral {
    ($target_type:ty) => {
        impl From<Peripheral> for $target_type {
            #[inline]
            fn from(val: Peripheral) -> $target_type {
                match val {
                    Peripheral::A => Self::A,
                    Peripheral::B => Self::B,
                    Peripheral::C => Self::C,
                    Peripheral::D => Self::D,
                    Peripheral::E => Self::E,
                    Peripheral::F => Self::F,
                    Peripheral::G => Self::G,
                    Peripheral::H => Self::H,
                }
            }
        }
    };
}
impl_from_peripheral!(atsamd21g::port::pmux0_::PMUXE_A);
impl_from_peripheral!(atsamd21g::port::pmux0_::PMUXO_A);
impl_from_peripheral!(atsamd21g::port::pmux1_::PMUXE_A);
impl_from_peripheral!(atsamd21g::port::pmux1_::PMUXO_A);


/// The Universal Magic I/O Pin Macro.
///
/// Examples of calls:
///
/// ```
/// let mut peripherals = atsamd21g::Peripherals::take().unwrap();
///
/// // Make PA01 and PA02 output pins.
/// iopin!(make_output, peripherals, PA, 1, 2);
///
/// // Make PA03 and PA04 input pins.
/// iopin!(make_input, peripherals, PA, 3, 4);
///
/// // Enable internal pull resistor on PA03, disable it on PA04.
/// iopin!(enable_pull, peripherals, PA, 3);
/// iopin!(disable_pull, peripherals, PA, 4);
///
/// // Set PA01 high and PA02 low.
/// // (If applied to input pins and the pull resistor is enabled,
/// // decides whether to pull up or down.)
/// iopin!(set_high, peripherals, PA, 1);
/// iopin!(set_low, peripherals, PA, 2);
///
/// // Take away PA05 from a peripheral.
/// iopin!(make_io, peripherals, PA, 5);
///
/// // Hand over PA06 to a peripheral.
/// iopin!(make_peripheral, peripherals, PA, 6);
///
/// // Select peripheral C for PA06.
/// iopin!(select_peripheral, peripherals, Peripheral::C, PA, 6);
/// ```
#[macro_export]
macro_rules! iopin {
    (make_output, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        // UNSAFE: there is no alternative to direct "bits" access and setting any bit in that
        // register leaves the SAM D21 in a valid state.
        unsafe {
            let val = iopin!(@indexes_as_bitmask, 0 $(, $pin_index)+);
            iopin!(@pinbank_to_dirset, $peripherals, $pinbank).modify(|_, w| w
                .dirset().bits(val)
            )
        };
    };
    (make_input, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        // UNSAFE: as above
        unsafe {
            let val = iopin!(@indexes_as_bitmask, 0 $(, $pin_index)+);
            iopin!(@pinbank_to_dirclr, $peripherals, $pinbank).modify(|_, w| w
                .dirclr().bits(val)
            )
        };
    };
    (enable_pull, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        $(
            iopin!(@pinbank_to_pincfg, $peripherals, $pinbank)[$pin_index].modify(|_, w| w
                .pullen().set_bit()
            );
        )+
    };
    (disable_pull, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        $(
            iopin!(@pinbank_to_pincfg, $peripherals, $pinbank)[$pin_index].modify(|_, w| w
                .pullen().clear_bit()
            );
        )+
    };
    (set_high, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        // UNSAFE: as above
        unsafe {
            let val = iopin!(@indexes_as_bitmask, 0 $(, $pin_index)+);
            iopin!(@pinbank_to_outset, $peripherals, $pinbank).modify(|_, w| w
                .outset().bits(val)
            )
        };
    };
    (set_low, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        // UNSAFE: as above
        unsafe {
            let val = iopin!(@indexes_as_bitmask, 0 $(, $pin_index)+);
            iopin!(@pinbank_to_outclr, $peripherals, $pinbank).modify(|_, w| w
                .outclr().bits(val)
            )
        };
    };
    (make_io, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        $(
            iopin!(@pinbank_to_pincfg, $peripherals, $pinbank)[$pin_index].modify(|_, w| w
                .pmuxen().clear_bit()
            );
        )+
    };
    (make_peripheral, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        $(
            iopin!(@pinbank_to_pincfg, $peripherals, $pinbank)[$pin_index].modify(|_, w| w
                .pmuxen().set_bit()
            );
        )+
    };
    (make_io, $peripherals:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        $(
            iopin!(@pinbank_to_pincfg, $peripherals, $pinbank)[$pin_index].modify(|_, w| w
                .pmuxen().clear_bit()
            );
        )+
    };
    (select_peripheral, $peripherals:expr, $peripheral:expr, $pinbank:ident $(, $pin_index:expr)+ $(,)?) => {
        $(
            if $pin_index % 2 == 0 {
                iopin!(@pinbank_to_pmux, $peripherals, $pinbank)[$pin_index/2].modify(|_, w| w
                    .pmuxe().variant($peripheral.into())
                )
            } else {
                iopin!(@pinbank_to_pmux, $peripherals, $pinbank)[$pin_index/2].modify(|_, w| w
                    .pmuxo().variant($peripheral.into())
                )
            }
        )+
    };

    // do not use any variant with the @ prefix, they are an internal implementation detail
    (@pinbank_to_dirset, $peripherals:expr, PA) => { $peripherals.PORT.dirset0 };
    (@pinbank_to_dirset, $peripherals:expr, PB) => { $peripherals.PORT.dirset1 };
    (@pinbank_to_dirclr, $peripherals:expr, PA) => { $peripherals.PORT.dirclr0 };
    (@pinbank_to_dirclr, $peripherals:expr, PB) => { $peripherals.PORT.dirclr1 };
    (@pinbank_to_outset, $peripherals:expr, PA) => { $peripherals.PORT.outset0 };
    (@pinbank_to_outset, $peripherals:expr, PB) => { $peripherals.PORT.outset1 };
    (@pinbank_to_outclr, $peripherals:expr, PA) => { $peripherals.PORT.outclr0 };
    (@pinbank_to_outclr, $peripherals:expr, PB) => { $peripherals.PORT.outclr1 };
    (@pinbank_to_pincfg, $peripherals:expr, PA) => { $peripherals.PORT.pincfg0_ };
    (@pinbank_to_pincfg, $peripherals:expr, PB) => { $peripherals.PORT.pincfg1_ };
    (@pinbank_to_pmux, $peripherals:expr, PA) => { $peripherals.PORT.pmux0_ };
    (@pinbank_to_pmux, $peripherals:expr, PB) => { $peripherals.PORT.pmux1_ };

    (@indexes_as_bitmask, $current_value:expr) => { $current_value };
    (@indexes_as_bitmask, $current_value:expr, $next_pin_index:expr $(, $more_pin_indexes:expr)*) => {
        iopin!(@indexes_as_bitmask, ($current_value | (1 << $next_pin_index)) $(, $more_pin_indexes)*)
    };
}
