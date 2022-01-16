extern crate minifb;

use minifb::{Key, Window, WindowOptions, ScaleMode, Scale};
use crate::sim::Simulation;
use std::time::{Duration, Instant};

/*use gif::{Encoder, Repeat, Frame};
use std::fs::File;
use chrono::Utc;

fn initGif() -> Encoder<File> {
    let dt = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut image = File::create(format!("gifs/{}.gif", dt)).unwrap();
    let mut encoder = Encoder::new(image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();
    
    encoder
}*/

pub mod util;
pub mod sim;

fn main() {
    let width = 512;
    let height = 512;
    
    let mut sim = Simulation::new(
        1.0,    // dA
        0.5,    // dB
        0.0150, // f
        0.0550, // k
        0.2,    // adj
        0.05,   // diag
        
        width, height
    );
    sim.seed();
    sim.generation();
    
    let mut window = Window::new("Reaction Diffusion", width, height, WindowOptions {
        borderless: false,
        title: true,
        resize: false,
        scale: Scale::X1,
        scale_mode: ScaleMode::AspectRatioStretch,
        topmost: false,
        transparency: false,
        none: false
    }).unwrap();
    window.limit_update_rate(Some(Duration::from_nanos(1)));
    
    //let mut last_time = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        /*let time = Instant::now().duration_since(last_time).as_secs_f64();
        println!("{:?}, {:?}", time, 1.0 / time);
        last_time = Instant::now();*/
        
        if window.is_key_down(Key::G) {
            sim.generation();
        }
        
        if !window.is_key_down(Key::Space) {
            window.update_with_buffer(&sim.framebuffer, width, height).unwrap();
        } else {
            window.update();
        }
    }
}
