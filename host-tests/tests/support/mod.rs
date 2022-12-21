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
        spi::Transaction::transfer(vec![command_byte], vec![0x0]),
        // send number of params
        spi::Transaction::transfer(vec![number_of_params], vec![0x0]),
        // send end command byte
        spi::Transaction::transfer(vec![0xee], vec![0x0]),
    ]
}

fn mock_response(command_byte: u8, number_of_params_to_receive: u8) -> Vec<spi::Transaction> {
    vec![
        // wait_response_cmd()
        // read start command
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        // read command byte | reply byte
        spi::Transaction::transfer(vec![command_reply_byte(command_byte)], vec![0xbf]),
        // read number of params to receive
        spi::Transaction::transfer(vec![number_of_params_to_receive], vec![0x1]),
        // test relies on max number of parameters being 8. This will probably change
        // as we understand more.
        spi::Transaction::transfer(vec![0xff], vec![0x8]),
        // read full 8 byte buffer
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
        // read end byte
        spi::Transaction::transfer(vec![0xff], vec![0xee]),
    ]
}

pub fn command_reply_byte(command: u8) -> u8 {
    command | 0x80
}
