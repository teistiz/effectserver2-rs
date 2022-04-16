use std::time::{Duration, Instant};

use effectserver2_rs::client::{UdpClient, LightParam};

fn main() {
    let addr = "localhost:9909";
    // let addr = "valot.instanssi:9909";
    let client = UdpClient::new(addr).expect("Unable to create UDP client!");

    let mut params: Vec<LightParam> = vec![];

    for index in 0..28 {
        params.push(LightParam(index, 8, 32, 64));
    }

    let started = Instant::now();
    loop {
        let elapsed = Instant::now().duration_since(started);
        let t = elapsed.as_secs_f32();

        let speed = 1.0;

        for index in 0..24 {
            let i = index as u8;
            let r = (t * speed + i as f32).sin() * 0.5 + 0.5;
            let g = (t * speed + i as f32 + 2.25).sin() * 0.5 + 0.5;
            let b = (t * speed + i as f32 + 4.5).sin() * 0.5 + 0.5;
            params[index] = LightParam::new_f(i, r * 0.5, g * 0.5, b * 0.5);
        }
        std::thread::sleep(Duration::from_millis(50));
        client.set("teistiz", &params);
    }

}
