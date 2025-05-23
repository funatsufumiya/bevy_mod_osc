use bevy::prelude::*;
use bevy_mod_osc::osc_receiver::{OscMessageEvent, OscReceiverPlugin};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        // NOTE: IPv6 is not compatible with IPv4, so you can't receive messages from IPv4
        //       if you want to make compatible with both, you should use IPv4 over IPv6 router or so on
        .add_plugins(OscReceiverPlugin {
            port: 1234,
            use_thread: true,
            use_ipv6: true,
            debug_print: true,
        })
        .add_systems(Update, osc_event_handler)
        .run();
}

fn osc_event_handler(
    mut events: EventReader<OscMessageEvent>,
) {
    for event in events.read() {
        if event.message.addr == "/test" {
            if event.message.args.len() < 1 {
                println!("/test received!");
            }else{
                let v = &event.message.args[0];
                match v {
                    rosc::OscType::Int(val) => {
                        println!("/test received! {}", val);
                    },
                    rosc::OscType::Float(val) => {
                        println!("/test received! {}", val);
                    },
                    rosc::OscType::String(val) => {
                        println!("/test received! {}", val);
                    },
                    _ => {
                        println!("/test received! {:?}", event.message.args);
                    }
                }
            }
        }
    }
}