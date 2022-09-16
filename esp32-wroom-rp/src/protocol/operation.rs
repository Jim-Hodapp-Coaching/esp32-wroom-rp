use super::protocol::{NinaCommand, NinaNoParams, NinaParam};

use heapless::Vec;
const MAX_NUMBER_OF_PARAMS: usize = 4;

pub struct Operation<P> {
    pub params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    pub command: NinaCommand,
    pub has_params: bool,
    pub number_of_params_to_receive: u8,
}

impl<P> Operation<P> {
    pub fn new(command: NinaCommand, number_of_params_to_receive: u8) -> Self {
        Self {
            params: Vec::new(),
            command: command,
            has_params: true,
            number_of_params_to_receive: number_of_params_to_receive,
        }
    }

    pub fn param(mut self, param: P) -> Self {
        self.params.push(param);
        self
    }

    pub fn with_no_params(mut self, no_param: P) -> Self {
        self.params.push(no_param);
        self.has_params = false;
        self
    }
}
