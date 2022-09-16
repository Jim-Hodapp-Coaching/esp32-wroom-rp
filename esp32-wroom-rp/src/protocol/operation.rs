use super::protocol::{NinaCommand, NinaNoParams, NinaParam};

use heapless::Vec;
const MAX_NUMBER_OF_PARAMS: usize = 4;

// Encapsulates all information needed to execute commands against Nina Firmware.
// along with user supplied data. Ex. SSID, passphrase, etc.

pub struct Operation<P> {
    pub params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    pub command: NinaCommand,
    pub has_params: bool,
    pub number_of_params_to_receive: u8,
}

impl<P> Operation<P> {
    // Initializes new Operation instance.
    //
    // `has_params` defaults to `true`
    pub fn new(command: NinaCommand, number_of_params_to_receive: u8) -> Self {
        Self {
            params: Vec::new(),
            command: command,
            has_params: true,
            number_of_params_to_receive: number_of_params_to_receive,
        }
    }

    // Pushes a new param into the internal `params` Vector.
    pub fn param(mut self, param: P) -> Self {
        self.params.push(param);
        self
    }

    // Used for denoting an Operation where no params are necessary.
    //
    // Sets `has_params` to `false`
    pub fn with_no_params(mut self, no_param: P) -> Self {
        self.params.push(no_param);
        self.has_params = false;
        self
    }
}
