The Arduino Zero offers three peripheral access methods available on the mikroBUS: SPI, I2C and
UART.

The canonical pinout is:

| Arduino pin | SAM D21 pin | Arduino usage   | SAM D21 SERCOM usage   |
| ----------- | ----------- | --------------- | ---------------------- |
| D0/RX       | PA11        | UART Rx         | SERCOM0/SERCOM2 PAD[3] |
| D1/TX       | PA10        | UART Tx         | SERCOM0/SERCOM2 PAD[2] |
| D11         | PA16        | SPI COPI        | SERCOM1/SERCOM3 PAD[0] |
| D12         | PA19        | SPI CIPO        | SERCOM1/SERCOM3 PAD[3] |
| D13         | PA17        | SPI SCK         | SERCOM1/SERCOM3 PAD[1] |
| D20/SDA     | PA22        | I2C SDA         | SERCOM3/SERCOM5 PAD[0] |
| D21/SCL     | PA23        | I2C SCL         | SERCOM3/SERCOM5 PAD[1] |
| (EDBG PA03) | PB22        | UART EDBG ← D21 | SERCOM5 PAD[2]         |
| (EDBG PA04) | PB23        | UART EDBG → D21 | SERCOM5 PAD[3]         |

This means that the relevant pins can only be controlled by specific SERCOM modules on the SAM D21:

| protocol  | SERCOMs          |
| --------- | ---------------- |
| UART      | SERCOM0, SERCOM2 |
| SPI       | SERCOM1, SERCOM3 |
| I2C       | SERCOM3, SERCOM5 |
| EDBG UART | SERCOM5          |

It is therefore prudent to assign the protocols as follows:

| protocol  | assigned SERCOM |
| --------- | --------------- |
| UART      | SERCOM0         |
| SPI       | SERCOM1         |
| I2C       | SERCOM3         |
| EDBG UART | SERCOM5         |

The following outgoing SPI data pinouts are supported by SAM D21:

| pinout | COPI   | SCK    | ~SS (if used) |
| ------ | ------ | ------ | ------------- |
| 0x0    | PAD[0] | PAD[1] | PAD[2]        |
| 0x1    | PAD[2] | PAD[3] | PAD[1]        |
| 0x2    | PAD[3] | PAD[1] | PAD[2]        |
| 0x3    | PAD[0] | PAD[3] | PAD[1]        |

To match the Arduino pinout, we choose pinout value 0x0.

The SAM D21 allows the choice of any pad of the SERCOM for incoming SPI data; to match the Arduino
pinout, we choose pad 3.
