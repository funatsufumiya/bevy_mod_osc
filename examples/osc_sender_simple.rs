use bevy::prelude::*;
use bevy_mod_osc::{osc_args, osc_sender::OscSender};
use rosc::OscType;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(OscSender::new("127.0.0.1", 1234))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut osc_sender: ResMut<OscSender>,
) {
    println!("Sending OSC messages to {}:{}", osc_sender.host, osc_sender.port);

    // osc send message
    osc_sender.send("/test", [1, 2, 3]);
    osc_sender.send("/test", osc_args!(1, 2.0, "a"));
    osc_sender.send("/test", vec![OscType::Int(3), OscType::Float(4.0), OscType::String("b".to_string())]);

    println!("Sent OSC message: /test 1 2 3");
    println!("Sent OSC message: /test 1 2.0 a");
    println!("Sent OSC message: /test 3 4.0 b");

    std::process::exit(0);
}