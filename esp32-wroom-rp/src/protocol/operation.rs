use crate::protocol::{NinaAbstractParam, NinaCommand};

use heapless::Vec;
const MAX_NUMBER_OF_PARAMS: usize = 4;

// Encapsulates all information needed to execute commands against Nina Firmware.
// along with user supplied data. Ex. SSID, passphrase, etc.

pub(crate) struct Operation<P> {
    pub params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    pub command: NinaCommand,
    pub has_params: bool,
    pub number_of_params_to_receive: u8,
}

impl Operation<NinaAbstractParam> {
    // Initializes new Operation instance.
    //
    // `has_params` defaults to `true`
    pub fn new(nina_command: NinaCommand, number_of_nina_params_to_receive: u8) -> Self {
        Self {
            params: Vec::new(),
            command: nina_command,
            has_params: false,
            number_of_params_to_receive: number_of_nina_params_to_receive,
        }
    }

    // Pushes a new param into the internal `params` Vector which
    // builds up an internal byte stream representing one Nina command
    // on the data bus.
    pub fn param(mut self, param: NinaAbstractParam) -> Self {
        self.params.push(param).ok().unwrap();
        self.has_params = true;
        self
    }
}
