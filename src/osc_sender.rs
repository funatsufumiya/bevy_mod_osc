use std::net::UdpSocket;
use bevy::prelude::*;
use rosc::{encoder, OscMessage, OscPacket, OscType};
// use crate::osc_arg; // Add missing import statement

#[derive(Resource)]
pub struct OscSender {
    /// The host to send OSC messages to (ex: 192.168.1.1)
    pub host: String,
    /// The port to send OSC messages to (ex: 1234)
    pub port: u16,
    /// Whether to use IPv6 (auto detect from host if use OscSender::new())
    pub ipv6: bool,
}

impl Default for OscSender {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1234,
            ipv6: false,
        }
    }
}

fn is_ipv6_addr(host: &str) -> bool {
    // WORKAROUND
    host.contains(":")
}

pub fn osc_arg<T: Into<OscType>>(arg: T) -> OscType {
    arg.into()
}

// define macro to given tuple as argument
#[macro_export] macro_rules! osc_args {
    ( $( $x:expr ),* ) => {
        vec![$( bevy_mod_osc::osc_sender::osc_arg($x) ),*]
    };
}

impl OscSender {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            ipv6: is_ipv6_addr(host),
        }
    }

    pub fn send<T, I>(&self, address: &str, args: T)
    where
        T: IntoIterator<Item = I>,
        I: Into<OscType>,
    {
        let from_ip = if self.ipv6 {
            "[::1]"
        } else {
            "0.0.0.0"
        };
        let client = UdpSocket::bind(format!("{}:0", from_ip)).expect("Failed to bind to socket");
        let packet = OscPacket::Message(OscMessage {
            addr: address.to_string(),
            args: args.into_iter().map(Into::into).collect(),
        });

        let buf = encoder::encode(&packet).unwrap();
        client.send_to(&buf, format!("{}:{}", self.host, self.port)).expect("Failed to send packet");
    }
}
