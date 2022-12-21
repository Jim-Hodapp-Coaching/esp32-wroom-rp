// use embedded_hal_mock::delay::MockNoop;
// use embedded_hal_mock::spi;

// use esp32_wroom_rp::network::{IpAddress, Port, TransportMode};
// use esp32_wroom_rp::tcp_client::{Connect, TcpClient};
// use esp32_wroom_rp::wifi::Wifi;

// pub mod support;

// use support::*;

// #[test]
// fn tcp_timeout_error() {
//     let mut spi_expectations =
//         // send_cmd() for start_client_tcp()
//         // send start byte
//         spi::Transaction::transfer(vec![0xe0], vec![0x0]),
//         // send command byte
//         spi::Transaction::transfer(vec![0x2e], vec![0x0]),
//         // send number of params
//         spi::Transaction::transfer(vec![0x0], vec![0x4]),
//         // send end command byte
//         spi::Transaction::transfer(vec![0xee], vec![0x0]),
//         // wait_response_cmd() for get_socket()
//         // read start command
//         spi::Transaction::transfer(vec![0xff], vec![0xe0]),
//         // read command byte | reply byte
//         spi::Transaction::transfer(vec![0xff], vec![0xbf]),
//         // read number of params to receive
//         spi::Transaction::transfer(vec![0xff], vec![0x1]),
//         // test relies on max number of parameters being 8. This will probably change
//         // as we understand more.
//         spi::Transaction::transfer(vec![0xff], vec![0x8]),
//         // read full 8 byte buffer
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         spi::Transaction::transfer(vec![0xff], vec![0xff]),
//         // read end byte
//         spi::Transaction::transfer(vec![0xff], vec![0xee]),
//         // // send_cmd() for start_client_tcp()
//         // spi::Transaction::transfer(vec![0xe0], vec![0x0]),
//         // spi::Transaction::transfer(vec![0x2e], vec![0x0]),
//         // spi::Transaction::transfer(vec![0x0], vec![0x0]),
//         // spi::Transaction::transfer(vec![0xee], vec![0x0]),
//         // // wait_response_cmd() for start_client_tcp()
//         // spi::Transaction::transfer(vec![0xff], vec![0xe0]),
//         // spi::Transaction::transfer(vec![0xff], vec![0xb7]),
//         // spi::Transaction::transfer(vec![0xff], vec![0x1]),
//         // // test relies on max number of parameters being 8. This will probably change
//         // // as we understand more.
//         // spi::Transaction::transfer(vec![0xff], vec![0x9]),
//     ];

//     // for _ in 0..10_000 {
//     //             // send_cmd() for get_client_state_tcp
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0xe0], vec![0x0]));
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0x2f], vec![0x0]));
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0x0], vec![0x0]));
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0xee], vec![0x0]));

//     //             // wait_response_cmd() for get_client_state_tcp
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xe0]));
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xb7]));
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x1]));
//     //             // test relies on max number of parameters being 8. This will probably change
//     //             // as we understand more.
//     //             spi_expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x1]));
//     // }

//     let spi = spi::Mock::new(&spi_expectations);

//     let mut delay = MockNoop::new();

//     let pins = EspControlMock {};

//     let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();

//     let ip_address: IpAddress = [0, 0, 0, 0];
//     let port: Port = 4000;
//     let mode: TransportMode = TransportMode::Tcp;

//     let result =
//         TcpClient::build(&mut wifi).connect(ip_address, port, mode, &mut delay, |_tcp_client| {});

//     assert_eq!(
//         result.unwrap_err(),
//         esp32_wroom_rp::Error::Tcp(esp32_wroom_rp::tcp_client::TcpError::Timeout)
//     );
// }
