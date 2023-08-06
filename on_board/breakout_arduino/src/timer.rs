use core::ptr::{read_volatile, write_volatile};
use core::time::Duration;

use cortex_m::Peripherals as CorePeripherals;
use cortex_m::asm::nop;
use cortex_m::peripheral::SYST;


static mut TICK_TIMER: u32 = 0;


/// Sets up the timer to raise an interrupt every 10 milliseconds.
pub(crate) fn set_up(core_peripherals: &mut CorePeripherals) {
    // initialize SysTick to trigger every 10ms
    core_peripherals.SYST.set_reload(SYST::get_ticks_per_10ms());
    core_peripherals.SYST.clear_current();
    core_peripherals.SYST.enable_interrupt();
    core_peripherals.SYST.enable_counter();
}

/// Increases the current timer value. Call only from `SysTick` interrupt handler!
#[inline]
pub(crate) fn tick() {
    let current_value = value();
    // UNSAFE: performed in an exception (interrupt) handler; race conditions are unlikely
    unsafe { write_volatile(&mut TICK_TIMER, current_value + 1) };
}

/// Returns the current timer value.
#[inline]
pub(crate) fn value() -> u32 {
    // UNSAFE: it's not an issue if we read the previous timer value every once in a while
    unsafe { read_volatile(&TICK_TIMER) }
}

/// Delays for the given duration.
pub(crate) fn delay(duration: Duration) {
    let tenms_units: u32 = (duration.as_millis() / 10).try_into().unwrap();
    let current_timer = value();
    let target_value = current_timer + tenms_units;

    while value() < target_value {
        nop();
    }
}
