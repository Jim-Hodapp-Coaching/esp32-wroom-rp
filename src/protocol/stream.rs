use super::protocol::{NinaCommand, NinaParam, NinaProtocolHandler, ProtocolInterface};
use crate::Error;

use heapless::Vec;

const MAX_NUMBER_OF_PARAMS: usize = 4;

pub struct Stream<BUS, CONTROL, P: NinaParam, I: ProtocolInterface<BUS, CONTROL>> {
    protocol_handler: I,
    params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    command: Option<NinaCommand>,
}

impl<BUS, CONTROL, P, I> Stream<BUS, CONTROL, P, I>
where
    P: NinaParam,
{
    fn new(protocol_handler: NinaProtocolHandler<BUS, CONTROL>) -> Self {
        Self {
            protocol_handler: protocol_handler,
            params: Vec::new(),
            command: None,
        }
    }

    fn command(self, command: NinaCommand) -> Self {
        Self {
            command: Some(command),
            ..self
        }
    }

    fn param(self, param: P) {
        self.params.push(param);
    }

    fn send(&mut self) -> Result<(), Error> {
        let params_iter = self.params.into_iter();
        let number_of_params: u8 = self.params.len();

        self.protocol_handler.control_pins.wait_for_esp_select();

        self.protocol_handler
            .send_cmd(self.command, number_of_params)
            .ok()
            .unwrap();

        // only send params if they are present
        if number_of_params > 0 {
            params_iter.for_each(|param| self.wifi.send_param(param));

            self.wifi.send_end_cmd();

            let param_size: u16 = params_iter.map(|param| param.length() as u16).sum();

            // This is to make sure we align correctly
            // 4 (start byte, command byte, reply byte, end byte) + the sum of all param lengths
            let command_size: u16 = 4u16 + param_size;
            self.pad_to_multiple_of_4(command_size);
        }

        self.protocol_handler.control_pins.esp_deselect();
    }

    fn wait_response(&mut self) {
        self.protocol.control_pins.wait_for_esp_select();

        let result = self
            .protocol_handler
            .wait_response_cmd(self.command, self.params.len());

        self.protocol_handler.control_pins.esp_deselect();

        result
    }
}
