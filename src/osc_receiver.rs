use std::{net::UdpSocket, task::Poll};
use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_async_task::TaskRunner;
use rosc::{OscMessage, OscPacket, OscType};
use std::collections::VecDeque;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref OSC_MESSAGE_QUEUE: Mutex<VecDeque<OscMessage>> = Mutex::new(VecDeque::new());
    static ref OSC_SOCKETS: Mutex<Vec<UdpSocket>> = Mutex::new(Vec::new());
}

pub struct OscReceiverPlugin {
    /// The port to receive OSC messages (ex: 1234)
    pub port: u16,
    /// Whether to use IPv6
    pub use_ipv6: bool,
    /// Whether to use thread. If false, it will use async task
    pub use_thread: bool,
    /// Whether to print debug messages
    pub debug_print: bool,
}

#[derive(Event)]
pub struct OscMessageEvent {
    pub message: OscMessage,
}

/// Message queue for threaded OSC handling
pub struct OscMessageQueue(pub VecDeque<OscMessage>);

#[derive(Resource)]
pub struct OscReceiver {
    /// The port to receive OSC messages (ex: 1234)
    pub port: u16,
    /// Whether to print debug messages
    pub debug_print: bool,
    pub socket: Option<UdpSocket>,
    pub using_ipv6: bool,
    // only used in threaded mode
    osc_message_queue: OscMessageQueue,
}

impl Default for OscReceiverPlugin {
    fn default() -> Self {
        Self {
            port: 1234,
            use_thread: true,
            use_ipv6: false,
            debug_print: false,
        }
    }
}

impl Plugin for OscReceiverPlugin {
    fn build(&self, app: &mut App) {
        let from_ip = if self.use_ipv6 {
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

        let is_first_time = !app.world().contains_resource::<Events<OscMessageEvent>>();

        app.add_event::<OscMessageEvent>();
        app.insert_resource(OscReceiver {
            port: self.port,
            debug_print: self.debug_print,
            socket: Some(socket),
            using_ipv6: self.use_ipv6,
            osc_message_queue: OscMessageQueue(VecDeque::new()),
        });

        // NOTE: register only once
            // println!("Registering OscMessageEvent");
        if self.use_thread {
            let mut world = app.world_mut();
            world.run_system_once(start_osc_handling_thread);
            if is_first_time {
                app.add_systems(Update, osc_handling_in_thread_update);
            }
        }else{
            if is_first_time {
                app.add_systems(Update, osc_handling_async);
            }
        }
    }
}

impl OscReceiverPlugin {
    pub fn new(port: u16, use_thread: bool, use_ipv6: bool, debug_print: bool) -> Self {
        Self {
            port,
            use_thread,
            use_ipv6,
            debug_print,
        }
    }

    pub fn new_ipv4(port: u16, use_thread: bool, debug_print: bool) -> Self {
        Self {
            port,
            use_thread,
            use_ipv6: false,
            debug_print,
        }
    }

    pub fn new_ipv6(port: u16, use_thread: bool, debug_print: bool) -> Self {
        Self {
            port,
            use_thread,
            use_ipv6: true,
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

fn handle_osc_message(msg: rosc::OscMessage, osc_message_queue: &mut OscMessageQueue, debug_print: bool) {
    if debug_print {
        debug_print_osc_message(&msg);
    }

    osc_message_queue.0.push_back(msg);
}

fn handle_osc_packet(packet: OscPacket, osc_message_queue: &mut OscMessageQueue, debug_print: bool) {
    match packet {
        OscPacket::Message(msg) => {
            handle_osc_message(msg, osc_message_queue, debug_print);
        }
        OscPacket::Bundle(bundle) => {
            bundle.content.iter().for_each(|packet| {
                handle_osc_packet(packet.clone(), osc_message_queue, debug_print);
            });
        }
    }
}

fn handle_osc_packet_in_thread(packet: OscPacket, message_queue: &mut VecDeque<OscMessage>, debug_print: bool) {
    match packet {
        OscPacket::Message(msg) => {
            if debug_print {
                debug_print_osc_message(&msg);
            }
            message_queue.push_back(msg);
        }
        OscPacket::Bundle(bundle) => {
            bundle.content.iter().for_each(|packet| {
                handle_osc_packet_in_thread(packet.clone(), message_queue, debug_print);
            });
        }
    }
}

async fn osc_handler(
    mut socket: UdpSocket,
    debug_print: bool,
) -> OscMessageQueue
{
    let mut buf = [0u8; rosc::decoder::MTU];
    let mut osc_message_queue = OscMessageQueue(VecDeque::new());
    match socket.recv_from(&mut buf) {
        Ok((size, _addr)) => {
            // println!("Received packet with size {} from: {}", size, addr);
            let packet = rosc::decoder::decode_udp(&buf[..size]).unwrap();
            handle_osc_packet(packet.1, &mut osc_message_queue, debug_print);
        }
        Err(e) => {
            warn!("Error receiving from socket: {}", e);
            // break;
        }
    }
    osc_message_queue
}

fn osc_handler_in_thread (
    mut socket: UdpSocket,
    debug_print: bool,
) -> VecDeque<OscMessage>
{
    let mut buf = [0u8; rosc::decoder::MTU];
    let mut osc_message_queue = VecDeque::new();
    match socket.recv_from(&mut buf) {
        Ok((size, _addr)) => {
            // println!("Received packet with size {} from: {}", size, addr);
            let packet = rosc::decoder::decode_udp(&buf[..size]).unwrap();
            handle_osc_packet_in_thread(packet.1, &mut osc_message_queue, debug_print);
        }
        Err(e) => {
            warn!("Error receiving from socket: {}", e);
            // break;
        }
    }
    osc_message_queue
}

/// communicate with OSC receiver in thread
pub fn osc_handling_in_thread_update (
    mut ev: EventWriter<OscMessageEvent>,
)
{
    let mut osc_message_queue = OSC_MESSAGE_QUEUE.lock().unwrap();
    for msg in osc_message_queue.iter() {
        ev.send(OscMessageEvent {
            message: msg.clone(),
        });
    }
    osc_message_queue.clear();
}

/// start OSC handling thread
pub fn start_osc_handling_thread (
    osc_receiver: Res<OscReceiver>,
    mut commands: Commands,
) {
    let n = OSC_SOCKETS.lock().unwrap().len();
    let debug_print = osc_receiver.debug_print;
    let osc_socket = osc_receiver.socket.as_ref().unwrap().try_clone().unwrap();
    OSC_SOCKETS.lock().unwrap().push(osc_socket);

    std::thread::spawn(move || {
        // println!("Starting OSC handling thread");
        loop {
            let osc_message_queue = osc_handler_in_thread (
                OSC_SOCKETS.lock().unwrap()[n].try_clone().unwrap(),
                debug_print
            );
            // OSC_MESSAGE_QUEUE.lock().unwrap().push_back(osc_message_queue);
            for msg in osc_message_queue {
                OSC_MESSAGE_QUEUE.lock().unwrap().push_back(msg);
            }
        }
    });
}

pub fn osc_handling_async(
    mut task_executor: TaskRunner<OscMessageQueue>,
    osc_receiver: Res<OscReceiver>,
    mut ev: EventWriter<OscMessageEvent>,
) {
    if task_executor.is_idle() {
        // NOTE: try_clone() is a workaround for borrowing issue. Probably no cost cloning a socket.
        task_executor.start(
            osc_handler(
                osc_receiver.socket.as_ref().unwrap().try_clone().unwrap(),
                osc_receiver.debug_print
            )
        );
    }

    match task_executor.poll() {
        Poll::Pending => {
            // println!("osc_handling: pending");
        }
        Poll::Ready(osc_message_queue) => {
            for msg in osc_message_queue.0 {
                ev.write(OscMessageEvent {
                    message: msg,
                });
            }
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