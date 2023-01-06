use heapless::Vec;

use super::{NinaAbstractParam, NinaCommand};

const MAX_NUMBER_OF_PARAMS: usize = 6;

// Encapsulates all information needed to execute commands against Nina Firmware.
// along with user supplied data. Ex. SSID, passphrase, etc.

pub(crate) struct Operation<P> {
    pub params: Vec<P, MAX_NUMBER_OF_PARAMS>,
    pub command: NinaCommand,
}

impl Operation<NinaAbstractParam> {
    // Initializes a new Operation instance with a specified command.
    pub fn new(nina_command: NinaCommand) -> Self {
        Self {
            params: Vec::new(),
            command: nina_command,
        }
    }

    // Pushes a new param into the internal `params` Vector which
    // builds up an internal byte stream representing one Nina command
    // on the data bus.
    pub fn param(mut self, param: NinaAbstractParam) -> Self {
        // FIXME: Vec::push() will return T when it is full, handle this gracefully
        self.params.push(param).ok().unwrap();
        self
    }
}
