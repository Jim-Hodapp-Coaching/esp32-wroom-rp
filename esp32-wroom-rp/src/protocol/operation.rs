use super::protocol::{NinaCommand, NinaParam, NinaProtocolHandler, ProtocolInterface};

use heapless::Vec;
const MAX_NUMBER_OF_PARAMS: usize = 4;

pub struct Operation<P> {
    params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    command: NinaCommand,
}

impl<P> Operation<P>
where
    P: NinaParam,
{
    pub fn new(command: NinaCommand) -> Self {
        Self {
            params: Vec::new(),
            command: command,
        }
    }

    pub fn param(mut self, param: P) -> Self {
        self.params.push(param);
        self
    }
}
