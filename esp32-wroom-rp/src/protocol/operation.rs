use super::protocol::{NinaCommand, NinaNoParams, NinaParam};

use heapless::Vec;
const MAX_NUMBER_OF_PARAMS: usize = 4;

pub struct Operation<P> {
    pub params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    pub command: NinaCommand,
    pub has_params: bool,
}

// try ?sized

impl<P> Operation<P> {
    pub fn new(command: NinaCommand) -> Self {
        Self {
            params: Vec::new(),
            command: command,
            has_params: true,
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
