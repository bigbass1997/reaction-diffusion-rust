#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate minifb;

use minifb::{Key, Window, WindowOptions, clamp};
use std::time::{Instant};
use std::mem::swap;

use oorandom;
use gif::{Encoder, Repeat, Frame};
use std::fs::File;
use chrono::Utc;
use ::f128::*;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;
const WMO: usize = WIDTH - 1;
const HMO: usize = HEIGHT - 1;

struct Cell {
    pub a: f128,
    pub b: f128,
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Cell{ a: self.a, b: self.b }
    }
}
impl Copy for Cell { }

fn from_f64_rgb(r: f64, g: f64, b: f64) -> u32 {
    (((r * 255.0) as u32) << 16) | (((g * 255.0) as u32) << 8) | ((b * 255.0) as u32)
}

fn from_f64_rgb_gray(v: f64) -> u32 {
    from_f64_rgb(v, v, v)
}

struct SimulationState {
    pub   dA: f128,
    pub   dB: f128,
    pub    f: f128,
    pub    k: f128,
    pub  adj: f128,
    pub diag: f128,
    
    pub curGrid: Vec<Cell>,
    pub nexGrid: Vec<Cell>,
}

fn initGif() -> Encoder<File> {
    let dt = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let mut image = File::create(format!("gifs/{}.gif", dt)).unwrap();
    let mut encoder = Encoder::new(image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();
    
    encoder
}

fn seedGrid(grid: &mut [Cell]) {
    let mut rng = oorandom::Rand64::new(4);
    let lo = (WIDTH * 4) as u64;
    let hi = ((WIDTH * HEIGHT) as u64) - lo;
    
    // random set of '+' seeds
    /*for i in 0..64 {
        let p = rng.rand_range(lo..hi) as usize;
        grid[p-1].b = 1.0;
        grid[p].b = 1.0;
        grid[p+1].b = 1.0;
        grid[p-WIDTH].b = 1.0;
        grid[p+WIDTH].b = 1.0;
    }*/
    
    // block in center
    for i in 127..129 {
        for j in 127..129 {
            grid[((j * WIDTH) + i) as usize].b = f128::ONE;
            //println!("{}, {}", i, j);
        }
    }
    
    // four corners?
    //grid[0].b = f128::ONE;
    //grid[WMO].b = f128::ONE;
    //grid[(HEIGHT - 1) * WIDTH].b = f128::ONE;
    //grid[(HEIGHT * WIDTH) - 1].b = f128::ONE;
}

fn main() {
    let mut sim = SimulationState {
        dA: f128::ONE,
        dB: f128!(0.5),
        f: f128!(0.055),
        k: f128!(0.062),
        adj: f128!(0.2),
        diag: f128!(0.05),
        
        curGrid: vec![Cell{a: f128!(1.0), b: f128!(0.0),}; WIDTH * HEIGHT],
        nexGrid: vec![Cell{a: f128!(1.0), b: f128!(0.0),}; WIDTH * HEIGHT],
    };
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];
    let mut gifbuf: [u8; WIDTH * HEIGHT * 3] = [0; WIDTH * HEIGHT * 3];
    let mut window = Window::new("Test", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| { panic!("{}", e); });
    //let mut encoder = initGif();
    
    seedGrid(&mut sim.curGrid);
    
    /* MAIN LOOP */
    let mut counter = 0;
    let mut total = 0;
    let mut lastTime = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let time = Instant::now().duration_since(lastTime).as_secs_f64();
        println!("{:?}, {:?}", time, 1.0 / time);
        lastTime = Instant::now();
        
        update(&mut sim, &mut buffer, &mut gifbuf);
        
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        
        /*if counter == 7 {
            encoder.write_frame(&Frame::from_rgb_speed(WIDTH as u16, HEIGHT as u16, &gifbuf, 30)).unwrap();
            counter = 0;
            total += 1;
            println!("{}", total);
        } else {
            counter += 1;
        }*/
    }
}

fn update(sim: &mut SimulationState, buf: &mut [u32], gif: &mut [u8]) {
    for (i, c) in &mut sim.curGrid.iter().enumerate() {
        let n = &mut sim.nexGrid[i];
        let abb = c.a * c.b * c.b;
        let (lapA, lapB) = laplacian(&sim.curGrid, i, sim.adj, sim.diag);
        
        n.a = clamp(f128!(0.0), c.a + (sim.dA * lapA) - abb + (sim.f * (f128!(1.0) - c.a)), f128!(1.0));
        n.b = clamp(f128!(0.0), c.b + (sim.dB * lapB) + abb - ((sim.k + sim.f) * c.b), f128!(1.0));
        
        let mut col = (n.a + n.b) * (n.a - n.b) * f128!(3.0);
        if col < f128!(0.0) {
            col = f128!(0.0);
        } else if col > f128!(1.0) {
            col = f128!(1.0);
        }
        
        buf[i] = from_f64_rgb_gray(col.into());
        //gif[(i * 3)    ] = (col * 255.0) as u8;
        //gif[(i * 3) + 1] = (col * 255.0) as u8;
        //gif[(i * 3) + 2] = (col * 255.0) as u8;
    }
    
    swap(&mut sim.curGrid, &mut sim.nexGrid);
}

fn laplacian(grid: &[Cell], i: usize, adj: f128, diag: f128) -> (f128, f128) {
    let x = i % WIDTH;
    let y = i / WIDTH;
    
    /* --------------- */
    let mut l = i;
    if x == 0 {
        l += WMO;
    } else {
        l -= 1;
    }
    
    let mut r = i;
    if x == WMO {
        r -= WMO;
    } else {
        r += 1;
    }
    
    let mut u = i;
    if y == 0 {
        u += WMO * HEIGHT;
    } else {
        u -= WIDTH;
    }
    
    let mut d = i;
    if y == HMO {
        d -= HMO * WIDTH;
    } else {
        d += WIDTH;
    }
    
    /* --------------- */
    
    let mut lu = u;
    if x == 0 {
        lu += WMO;
    } else {
        lu -= 1;
    }
    
    let mut ru = u;
    if x == WMO {
        ru -= WMO;
    } else {
        ru += 1;
    }
    
    let mut ld = d;
    if x == 0 {
        ld += WMO;
    } else {
        ld -= 1;
    }
    
    let mut rd = d;
    if x == WMO {
        rd -= WMO;
    } else {
        rd += 1;
    }
    /* --------------- */
    
    let lapA = -grid[i].a + ((grid[l].a + grid[r].a + grid[u].a + grid[d].a) * adj)
                          + ((grid[lu].a + grid[ru].a + grid[ld].a + grid[rd].a) * diag);
    
    let lapB = -grid[i].b + ((grid[l].b + grid[r].b + grid[u].b + grid[d].b) * adj)
                          + ((grid[lu].b + grid[ru].b + grid[ld].b + grid[rd].b) * diag);
    (lapA, lapB)
}