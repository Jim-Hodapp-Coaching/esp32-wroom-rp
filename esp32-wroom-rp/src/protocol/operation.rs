use super::protocol::{NinaCommand, NinaParam};

use heapless::Vec;
const MAX_NUMBER_OF_PARAMS: usize = 4;
pub struct Operation<P> {
    pub params: Option<Vec<P, MAX_NUMBER_OF_PARAMS>>,
    pub command: NinaCommand,
}

impl<P> Operation<P> {
    pub fn new(command: NinaCommand) -> Self {
        Self {
            params: Some(Vec::new()),
            command: command,
        }
    }

    pub fn param(mut self, param: P) -> Self {
        self.params.unwrap().push(param);
        self
    }
}
