use embedded_hal_mock::spi;
use esp32_wroom_rp::gpio::EspControlInterface;

pub(crate) struct EspControlMock {}

impl EspControlInterface for EspControlMock {
    fn init(&mut self) {}

    fn reset<D>(&mut self, _delay: &mut D) {}

    fn get_esp_ack(&self) -> bool {
        true
    }

    fn wait_for_esp_select(&mut self) {}

    fn wait_for_esp_ack(&self) {}

    fn wait_for_esp_ready(&self) {}

    fn esp_select(&mut self) {}

    fn esp_deselect(&mut self) {}

    fn get_esp_ready(&self) -> bool {
        true
    }
}

pub fn mock_command(command_byte: u8, number_of_params: u8) -> Vec<spi::Transaction> {
    vec![
        // send_cmd()
        // send start byte
        spi::Transaction::transfer(vec![0xe0], vec![0x0]),
        // send command byte
        spi::Transaction::transfer(vec![command_and_reply_byte(command_byte)], vec![0x0]),
        // send number of params
        spi::Transaction::transfer(vec![number_of_params], vec![0x0]),
    ]
}

pub fn mock_single_byte_size_params(
    number_of_param_bytes: u8,
    byte_value: u8,
) -> Vec<spi::Transaction> {
    let mut expectations = vec![spi::Transaction::transfer(
        vec![number_of_param_bytes],
        vec![0x0],
    )];

    for _ in 0..number_of_param_bytes {
        expectations.push(spi::Transaction::transfer(vec![byte_value], vec![0x0]));
    }

    expectations
}

pub fn mock_padding(number_of_padding_bytes: u8) -> Vec<spi::Transaction> {
    let mut expectations = Vec::new();
    for _ in 0..number_of_padding_bytes {
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x0]));
    }

    expectations
}

pub fn mock_end_byte() -> Vec<spi::Transaction> {
    vec![
        // send end command byte
        spi::Transaction::transfer(vec![0xee], vec![0x0]),
    ]
}

pub fn mock_receive(
    command_byte: u8,
    number_of_params_to_receive: u8,
    values_to_receive: &[u8],
) -> Vec<spi::Transaction> {
    // let mut buffer = vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x21];
    // Size 4096 should match MAX_NINA_RESPONSE_LENGTH
    let mut buffer = vec![0xff; 4096];
    buffer[7] = 0x21;

    let length_of_values = if values_to_receive.len() > 0 {
        values_to_receive.len() - 1
    } else {
        0
    };

    // replace buffer values with values from values_to_receive
    buffer.splice(0..length_of_values, values_to_receive.iter().cloned());

    let mut expectations = vec![
        // wait_response_cmd()
        // read start command
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        // read command byte | reply byte
        spi::Transaction::transfer(vec![0xff], vec![command_or_reply_byte(command_byte)]),
        // read number of params to receive
        spi::Transaction::transfer(vec![0xff], vec![number_of_params_to_receive]),
        // test relies on max number of parameters being 8. This will probably change
        // as we understand more.
        spi::Transaction::transfer(vec![0xff], vec![0x8]),
    ];

    for byte in buffer.iter().cloned() {
        expectations.append(&mut vec![spi::Transaction::transfer(
            vec![0xff],
            vec![byte],
        )]);
        // expectations.push(spi::Transaction::transfer(vec![0xff], vec![byte]));
    }
    expectations
}

pub fn command_or_reply_byte(command: u8) -> u8 {
    command | 0x80
}

pub fn command_and_reply_byte(command: u8) -> u8 {
    (command as u8) & !(0x80 as u8)
}
