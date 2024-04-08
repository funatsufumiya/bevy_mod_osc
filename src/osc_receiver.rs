use std::net::UdpSocket;
use bevy::{ecs::system::CommandQueue, log::tracing_subscriber::field::debug, prelude::*, scene::ron::de};
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use rosc::{OscMessage, OscPacket, OscType};

pub struct OscReceiverPlugin {
    /// The port to receive OSC messages (ex: 1234)
    pub port: u16,
    /// Whether to use IPv6
    pub ipv6: bool,
    /// Whether to print debug messages
    pub debug_print: bool,
}

#[derive(Event)]
pub struct OscMessageEvent {
    pub message: OscMessage,
}

#[derive(Resource)]
pub struct OscReceiver {
    /// The port to receive OSC messages (ex: 1234)
    pub port: u16,
    /// Whether to print debug messages
    pub debug_print: bool,
    pub socket: Option<UdpSocket>,
    pub ipv6: bool,
}

impl Default for OscReceiverPlugin {
    fn default() -> Self {
        Self {
            port: 1234,
            ipv6: false,
            debug_print: false,
        }
    }
}

impl Plugin for OscReceiverPlugin {
    fn build(&self, app: &mut App) {
        let from_ip = if self.ipv6 {
            "[::1]"
        } else {
            "0.0.0.0"
        };
        let socket = UdpSocket::bind(format!("{}:{}",
            from_ip, self.port
        )).expect("Failed to bind to socket");

        if self.debug_print {
            println!("Listening for OSC on {}:{}", from_ip, self.port);
        }

        let is_first_time = !app.world.contains_resource::<Events<OscMessageEvent>>();

        app.add_event::<OscMessageEvent>();
        app.insert_resource(OscReceiver {
            port: self.port,
            debug_print: self.debug_print,
            socket: Some(socket),
            ipv6: self.ipv6,
        });

        // NOTE: register only once
        if is_first_time {
            // println!("Registering OscMessageEvent");
            app.add_systems(Update, osc_handling_async);
        }
    }
}

impl OscReceiverPlugin {
    pub fn new(port: u16, ipv6:bool, debug_print: bool) -> Self {
        Self {
            port,
            ipv6,
            debug_print,
        }
    }

    pub fn new_ipv4(port: u16, debug_print: bool) -> Self {
        Self {
            port,
            ipv6: false,
            debug_print,
        }
    }

    pub fn new_ipv6(port: u16, debug_print: bool) -> Self {
        Self {
            port,
            ipv6: true,
            debug_print,
        }
    }
}

fn debug_print_osc_message(msg: &OscMessage) {
    let time_str = chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S%.6f").to_string();
    if msg.args.len() == 0 {
        println!("[{}] Received OSC Message: {} (type tags: '')", time_str, msg.addr);
        return;
    }
    println!("[{}] Received OSC Message: {} {} (type tags: '{}')", time_str, msg.addr,
        &msg.args.iter().map(|arg| get_string(arg)).collect::<Vec<String>>().join(" "),
        get_type_tags(&msg.args.iter().collect::<Vec<_>>()));
}

fn handle_osc_message(msg: rosc::OscMessage, command_queue: &mut CommandQueue, debug_print: bool) {
    if debug_print {
        debug_print_osc_message(&msg);
    }

    command_queue.push(move |world: &mut World| {
        world.send_event(OscMessageEvent {
            message: msg.clone(),
        });
    });
}

fn handle_osc_packet(packet: OscPacket, command_queue: &mut CommandQueue, debug_print: bool) {
    match packet {
        OscPacket::Message(msg) => {
            handle_osc_message(msg, command_queue, debug_print);
        }
        OscPacket::Bundle(bundle) => {
            bundle.content.iter().for_each(|packet| {
                handle_osc_packet(packet.clone(), command_queue, debug_print);
            });
        }
    }
}

async fn osc_handler(
    mut socket: UdpSocket,
    debug_print: bool,
) -> CommandQueue
{
    let mut buf = [0u8; rosc::decoder::MTU];
    let mut command_queue = CommandQueue::default();
    match socket.recv_from(&mut buf) {
        Ok((size, _addr)) => {
            // println!("Received packet with size {} from: {}", size, addr);
            let packet = rosc::decoder::decode_udp(&buf[..size]).unwrap();
            handle_osc_packet(packet.1, &mut command_queue, debug_print);
        }
        Err(e) => {
            warn!("Error receiving from socket: {}", e);
            // break;
        }
    }
    command_queue
}

pub fn osc_handling_async(
    mut task_executor: AsyncTaskRunner<CommandQueue>,
    osc_receiver: Res<OscReceiver>,
    mut commands: Commands,
) {
    match task_executor.poll() {
        AsyncTaskStatus::Idle => {
            // NOTE: try_clone() is a workaround for borrowing issue. Probably no cost cloning a socket.
            task_executor.start(
                osc_handler(
                    osc_receiver.socket.as_ref().unwrap().try_clone().unwrap(),
                    osc_receiver.debug_print
                )
            );
        }
        AsyncTaskStatus::Pending => {
            // println!("osc_handling: pending");
        }
        AsyncTaskStatus::Finished(command_queue) => {
            let mut command_queue = command_queue;
            commands.append(&mut command_queue);
        }
    }
}

fn get_type_string(osc_type: &OscType) -> String {
    match osc_type {
        OscType::Int(_) => "i".to_string(),
        OscType::Float(_) => "f".to_string(),
        OscType::Double(_) => "f".to_string(),
        OscType::String(_) => "s".to_string(),
        OscType::Blob(_) => "b".to_string(),
        OscType::Bool(_) => "i".to_string(),
        default => panic!("Unsupported type: {:?}", default),
    }
}

fn get_string(osc_type: &OscType) -> String {
    match osc_type {
        OscType::Int(i) => i.to_string(),
        OscType::Float(f) => f.to_string(),
        OscType::Double(d) => d.to_string(),
        OscType::String(s) => s.to_string(),
        OscType::Blob(b) => format!("{:?}", b),
        OscType::Bool(b) => b.to_string(),
        default => panic!("Unsupported type: {:?}", default),
    }
}

fn get_type_tags(args: &Vec<&OscType>) -> String {
    args.iter().map(|arg| get_type_string(arg)).collect::<Vec<String>>().join("")
}