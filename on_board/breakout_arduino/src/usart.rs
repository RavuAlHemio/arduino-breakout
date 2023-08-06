use atsamd21g::Peripherals;


pub(crate) fn write(peripherals: &mut Peripherals, buf: &[u8]) {
    let sercom5_usart = peripherals.SERCOM5.usart();

    // first, wait for shift register to empty
    while sercom5_usart.intflag.read().dre().bit_is_clear() {
    }

    for b in buf {
        unsafe {
            sercom5_usart.data.modify(|_, w| w
                .data().bits((*b).into())
            )
        };
        while sercom5_usart.intflag.read().dre().bit_is_clear() {
        }
    }

    while sercom5_usart.intflag.read().txc().bit_is_clear() {
    }
}
