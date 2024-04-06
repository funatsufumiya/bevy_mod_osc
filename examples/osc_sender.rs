use bevy::prelude::*;
use bevy_mod_osc::osc_sender::OscSender;
use std::time::Duration;

#[derive(Resource)]
struct LastSentTime {
    pub last_sent_time: Option<Duration>
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(OscSender::new("127.0.0.1", 1234))
        .insert_resource(LastSentTime {
            last_sent_time: None
        })
        .add_systems(Update, update)
        .run();
}

fn update(
    mut osc_sender: ResMut<OscSender>,
    time: Res<Time>,
    mut last_sent_time: ResMut<LastSentTime>
) {
    // send each second
    if last_sent_time.last_sent_time.is_none() || last_sent_time.last_sent_time.unwrap() + Duration::from_secs(1) < time.elapsed() {
        let val: i32 = (time.elapsed_seconds().sin() * 100.0) as i32;

        // osc send message
        osc_sender.send("/test", vec![rosc::OscType::Int(val)]);

        println!("Sent OSC message: /test {} to {}:{}", val, osc_sender.host, osc_sender.port);

        last_sent_time.last_sent_time = Some(time.elapsed());
    }
}