use super::{Error, Params, FirmwareVersion, IoInterface, NinaCommandHandler, NinaCommand};

struct SpiCommandHandler<I: IoInterface> {
    io_interface: I
}

impl<I: IoInterface> SpiCommandHandler<I> {
    fn send_command(&self, command: NinaCommand, parameters: [u8; 5]) -> Result<FirmwareVersion, Error> {
        Ok(FirmwareVersion::new([0x31,0x2e,0x37,0x2e,0x34])) // 1.7.4
      }
}

impl<I: IoInterface> NinaCommandHandler for SpiCommandHandler<I> {

    fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, Error> {
        self.send_command(NinaCommand::StartClientTcp, params)
    }

    fn get_fw_version(&self) -> Result<FirmwareVersion, Error> {
        self.send_command(NinaCommand::GetFirmwareVersion, [0; 5])
    }
}