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