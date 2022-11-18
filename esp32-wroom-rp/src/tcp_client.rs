use super::protocol::NinaProtocolHandler;

use super::IpAddress;

pub struct TcpClient<'a, 'b, B, C> {
    pub(crate) common: TcpClientCommon<'a, NinaProtocolHandler<'a, B, C>>,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) server_hostname: Option<&'b str>
}

pub(crate) struct TcpClientCommon<'a, PH> {
    pub(crate) protocol_handler: &'a PH,
}

impl<'a, 'b, B, C> TcpClient<'a, 'b, B, C> {
    fn server_ip_address(&mut self, ip: IpAddress) {
        self.server_ip_address = Some(ip);
    }

    fn server_hostname(&mut self, hostname: &'b str) {
        self.server_hostname = Some(hostname);
    }
}
