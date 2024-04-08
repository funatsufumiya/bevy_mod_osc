use std::net::UdpSocket;
use bevy::prelude::*;
use rosc::{encoder, OscArray, OscMessage, OscPacket, OscType};

#[derive(Resource)]
pub struct OscSender {
    /// The host to send OSC messages to (ex: 192.168.1.1)
    pub host: String,
    /// The port to send OSC messages to (ex: 1234)
    pub port: u16,
}

impl Default for OscSender {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 1234,
        }
    }
}

impl OscSender {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    pub fn send(&self, address: &str, args: Vec<OscType>) {
        let client = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to socket");
        let packet = OscPacket::Message(OscMessage {
            addr: address.to_string(),
            args: args,
        });

        let buf = encoder::encode(&packet).unwrap();
        client.send_to(&buf, format!("{}:{}", self.host, self.port)).expect("Failed to send packet");
    }
}
