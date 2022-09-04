use super::protocol::{NinaCommand, NinaParam, NinaProtocolHandler, ProtocolInterface};
use crate::gpio::EspControlInterface;
use crate::Error;

use heapless::Vec;

const MAX_NUMBER_OF_PARAMS: usize = 4;

pub struct Stream<'a, P: NinaParam, I: ProtocolInterface> {
    protocol_handler: &'a mut I,
    params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    command: Option<NinaCommand>,
}

impl<'a, P, I> Stream<'a, P, I>
where
    P: NinaParam,
    I: ProtocolInterface,
{
    pub fn new(protocol_handler: &mut I) -> Self {
        Self {
            protocol_handler: protocol_handler,
            params: Vec::new(),
            command: None,
        }
    }

    pub fn command(self, command: NinaCommand) -> Self {
        Self {
            command: Some(command),
            ..self
        }
    }

    pub fn param(self, param: P) {
        self.params.push(param);
    }

    pub fn send(&mut self) -> Result<(), Error> {
        let params_iter = self.params.into_iter();
        let number_of_params: u8 = self.params.len() as u8;
        let control_pins = self.protocol_handler.control_pins;
        let Some(command) = self.command;

        control_pins.wait_for_esp_select();

        self.protocol_handler
            .send_cmd(command, number_of_params)
            .ok()
            .unwrap();

        // only send params if they are present
        if number_of_params > 0 {
            params_iter.for_each(|param| self.protocol_handler.send_param(param));

            self.protocol_handler.send_end_cmd();

            let param_size: u16 = params_iter.map(|param| param.length()).sum();

            // This is to make sure we align correctly
            // 4 (start byte, command byte, reply byte, end byte) + the sum of all param lengths
            let command_size: u16 = 4u16 + param_size;
            self.protocol_handler.pad_to_multiple_of_4(command_size);
        }

        control_pins.esp_deselect();
    }

    pub fn wait_response(&mut self) {
        let control_pins = self.protocol_handler.into_control_interface();
        let Some(command) = self.command;

        control_pins.wait_for_esp_select();

        let result = self
            .protocol_handler
            .wait_response_cmd(command, self.params.len() as u8);

        control_pins.esp_deselect();

        result
    }
}
