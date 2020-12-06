use rand::seq::SliceRandom;
use rand::Rng;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::convert::TryInto;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Colour<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl<T> Colour<T>
where
    T: Copy + From<u8>,
{
    pub fn new(r: T, g: T, b: T) -> Self {
        Colour { r, g, b }
    }

    pub fn to_array(&self) -> [T; 4] {
        [self.r, self.g, self.b, 0u8.into()]
    }
}

const NUM_LEDS: usize = 80;

pub fn wipe(controller: &mut Controller, delay: u64) -> Result<(), Box<dyn std::error::Error>> {
    let leds = controller.leds_mut(0);
    for led in leds.iter_mut() {
        *led = [0, 0, 0, 0]
    }

    controller.render()?;

    for i in 0..NUM_LEDS {
        thread::sleep(Duration::from_millis(delay));

        let leds = controller.leds_mut(0);
        for led in leds.iter_mut().take(i + 1) {
            *led = [255, 255, 255, 0]
        }
        controller.render()?;
    }
    for i in 0..NUM_LEDS {
        thread::sleep(Duration::from_millis(delay));
        for n in 0..(i + 1) {
            let leds = controller.leds_mut(0);
            leds[NUM_LEDS - 1 - n] = [0, 0, 0, 0]
        }
        controller.render()?;
    }
    Ok(())
}

fn wheel(pos: u8) -> [u8; 4] {
    if pos < 85 {
        [pos * 3, 255 - pos * 3, 0, 0]
    } else if pos < 170 {
        [255 - (pos - 85) * 3, 0, (pos - 85) * 3, 0]
    } else {
        [0, (pos - 170) * 3, 255 - (pos - 170) * 3, 0]
    }
}

pub fn theatre(
    controller: &mut Controller,
    j: u8,
    delay: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let leds = controller.leds_mut(0);

    for (i, led) in leds.iter_mut().enumerate() {
        let j: usize = j.into();
        if (i + j) % 3 == 0 {
            *led = [255, 255, 255, 0];
        } else {
            *led = [0, 0, 0, 0];
        }
    }
    controller.render()?;
    thread::sleep(Duration::from_millis(delay));

    Ok(())
}

pub fn rainbow(
    controller: &mut Controller,
    j: u8,
    delay: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let leds = controller.leds_mut(0);

    for (i, led) in leds.iter_mut().enumerate() {
        let j: usize = j.into();
        let pinx = (i * 256 / NUM_LEDS) + j;
        *led = wheel((pinx & 255).try_into()?);
    }
    controller.render()?;
    thread::sleep(Duration::from_millis(delay));

    Ok(())
}

pub fn bands(
    controller: &mut Controller,
    j: u8,
    inner: bool,
    num_in_band: usize,
    delay: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let leds = controller.leds_mut(0);

    for (i, led) in leds.iter_mut().enumerate() {
        let ju: usize = j.try_into().unwrap();
        let switch = if inner {
            ((i + ju) / num_in_band) % 2 == 0
        } else {
            ((i / num_in_band) + ju) % 2 == 0
        };
        if switch {
            *led = [255, 0, 0, 0];
        } else {
            *led = [0, 255, 0, 0];
        }
    }
    controller.render()?;
    thread::sleep(Duration::from_millis(delay));

    Ok(())
}

pub fn colour(leds: &mut [[u8; 4]], colour: Colour<u8>) {
    for led in leds.iter_mut() {
        *led = colour.to_array();
    }

    thread::sleep(Duration::from_secs(1));
}

pub fn rainbow_explode(controller: &mut Controller) -> Result<(), Box<dyn std::error::Error>> {
    for j in 0..10 {
        let leds = controller.leds_mut(0);
        for led in leds.iter_mut() {
            *led = [0, 0, 0, 0];
        }

        leds[0] = [j * 20, j * 20, j * 20, 0];

        controller.render()?;
        thread::sleep(Duration::from_millis(75));
    }

    for j in 0..NUM_LEDS {
        let leds = controller.leds_mut(0);

        for led in leds.iter_mut() {
            *led = [0, 0, 0, 0];
        }

        let pos = j % NUM_LEDS;
        leds[pos] = [255, 255, 255, 0];

        let tail_len = 10u8;
        for b in 1..(tail_len) {
            // May not be correct around the ends of the strip
            let pos = (pos.wrapping_sub(b.into())) % NUM_LEDS;

            // Start at lower to make more distinct
            let vr = 200 / (b + 1);
            let vg = 200 / (b + 1);
            let vb = 200 / (b + 1);

            leds[pos] = [vr, vg, vb, 0];
        }

        controller.render()?;
        let a: u64 = (NUM_LEDS - j).try_into().unwrap();
        let speed_factor = a / 4 + 5;
        thread::sleep(Duration::from_millis(speed_factor));
    }

    {
        let leds = controller.leds_mut(0);

        for j in 0..(NUM_LEDS / 7) {
            leds[NUM_LEDS - 1 - j] = [10, 10, 10, 0];
        }

        controller.render()?;
        thread::sleep(Duration::from_millis(75));
    }

    for j in 1..(NUM_LEDS / 7) + 1 {
        let leds = controller.leds_mut(0);
        for led in leds.iter_mut() {
            *led = [0, 0, 0, 0];
        }

        // ROYGBIV
        // R is NUM_LEDS-1 to NUM_LEDS-
        for k in 0..j {
            let j8: u8 = j.try_into().unwrap();
            let j8 = j8 / 2 + 1;
            leds[NUM_LEDS - 1 - k] = [255 / j8, 0, 0, 0];
            leds[NUM_LEDS - 1 - k - j] = [250 / j8, 150 / j8, 0, 0];
            leds[NUM_LEDS - 1 - k - (j * 2)] = [250 / j8, 250 / j8, 0, 0];
            leds[NUM_LEDS - 1 - k - (j * 3)] = [0, 250 / j8, 0, 0];
            leds[NUM_LEDS - 1 - k - (j * 4)] = [0, 0, 250 / j8, 0];
            leds[NUM_LEDS - 1 - k - (j * 5)] = [50 / j8, 0, 250 / j8, 0];
            leds[NUM_LEDS - 1 - k - (j * 6)] = [250 / j8, 0, 250 / j8, 0];
        }

        controller.render()?;
        let speed_factor: u64 = (j * 50).try_into().unwrap();
        thread::sleep(Duration::from_millis(50 + speed_factor));
    }

    for j in (NUM_LEDS / 7)..(NUM_LEDS / 7) + 6 {
        let leds = controller.leds_mut(0);
        for led in leds.iter_mut() {
            *led = [0, 0, 0, 0];
        }

        // ROYGBIV
        for k in 0..(NUM_LEDS / 7) {
            let j8: u8 = j.try_into().unwrap();
            leds[NUM_LEDS - 1 - k] = [255 / j8, 0, 0, 0];
            leds[NUM_LEDS - 1 - k - (NUM_LEDS / 7)] = [250 / j8, 150 / j8, 0, 0];
            leds[NUM_LEDS - 1 - k - ((NUM_LEDS / 7) * 2)] = [250 / j8, 250 / j8, 0, 0];
            leds[NUM_LEDS - 1 - k - ((NUM_LEDS / 7) * 3)] = [0, 250 / j8, 0, 0];
            leds[NUM_LEDS - 1 - k - ((NUM_LEDS / 7) * 4)] = [0, 0, 250 / j8, 0];
            leds[NUM_LEDS - 1 - k - ((NUM_LEDS / 7) * 5)] = [50 / j8, 0, 250 / j8, 0];
            leds[NUM_LEDS - 1 - k - ((NUM_LEDS / 7) * 6)] = [250 / j8, 0, 250 / j8, 0];
        }

        controller.render()?;
        thread::sleep(Duration::from_millis(150));
    }
    Ok(())
}

pub fn tracer(
    controller: &mut Controller,
    j: u8,
    c1: (u8, u8, u8),
    c2: (u8, u8, u8),
) -> Result<(), Box<dyn std::error::Error>> {
    let leds = controller.leds_mut(0);

    for led in leds.iter_mut() {
        *led = [0, 0, 0, 0];
    }

    // If max step is not a multiple of the strip length then
    // the pattern may not loop smoothly
    let ju: usize = j.try_into().unwrap();
    let pos = ju % NUM_LEDS;
    leds[pos] = [c1.0, c1.1, c1.2, 0];

    let tail_len = 10u8;
    for b in 1..(tail_len) {
        // May not be correct around the ends of the strip
        let pos = (pos.wrapping_sub(b.into())) % NUM_LEDS;

        // Start at lower to make more distinct
        let vr = c1.0.saturating_sub(50).saturating_sub((255 / tail_len) * b);
        let vg = c1.1.saturating_sub(50).saturating_sub((255 / tail_len) * b);
        let vb = c1.2.saturating_sub(50).saturating_sub((255 / tail_len) * b);

        leds[pos] = [vr, vg, vb, 0];
    }

    let pos = (ju + NUM_LEDS / 2) % NUM_LEDS;
    leds[pos] = [c2.0, c2.1, c2.2, 0];

    let tail_len = 10u8;
    for b in 1..(tail_len) {
        // May not be correct around the ends of the strip
        let pos = (pos.wrapping_sub(b.into())) % NUM_LEDS;

        // Start at lower to make more distinct
        let vr = c2.0.saturating_sub(50).saturating_sub((255 / tail_len) * b);
        let vg = c2.1.saturating_sub(50).saturating_sub((255 / tail_len) * b);
        let vb = c2.2.saturating_sub(50).saturating_sub((255 / tail_len) * b);

        leds[pos] = [vr, vg, vb, 0];
    }

    controller.render()?;
    thread::sleep(Duration::from_millis(50));

    Ok(())
}

struct Element {
    index: usize,
    step: u8,
}

pub fn random(controller: &mut Controller) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let mut to_light = Vec::new();
    let mut to_drop = Vec::new();
    let mut vec2 = Vec::new();
    let mut vec3 = Vec::new();
    let mut lit = Vec::new();
    let mut processing = Vec::new();

    loop {
        let leds = controller.leds_mut(0);
        if processing.len() < 20 {
            let to_light_id = rng.gen_range(0, NUM_LEDS);
            if !processing.contains(&to_light_id) {
                to_light.push(Element {
                    index: to_light_id,
                    step: 0,
                });
                processing.push(to_light_id);
            }
        }

        if lit.len() >= 10 {
            lit.shuffle(&mut rng);
            let id = lit.pop().unwrap();

            to_drop.push(Element {
                index: id,
                step: 250,
            });
        }

        for e in &to_drop {
            let e2 = Element {
                index: e.index,
                step: e.step - 25,
            };
            leds[e2.index] = [e2.step, e2.step, e2.step, 0];
            if e2.step != 0 {
                vec3.push(e2);
            } else {
                let idx = processing.iter().position(|z| *z == e2.index).unwrap();
                processing.remove(idx);
            }
        }

        for e in &to_light {
            let e2 = Element {
                index: e.index,
                step: e.step + 25,
            };
            leds[e2.index] = [e2.step, e2.step, e2.step, 0];
            if e2.step >= 250 {
                lit.push(e2.index);
            } else {
                vec2.push(e2);
            }
        }

        to_light.clear();
        to_light.append(&mut vec2);
        to_drop.clear();
        to_drop.append(&mut vec3);

        controller.render()?;
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn random_col(
    controller: &mut Controller,
    pastel: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let mut to_light = Vec::new();
    let mut to_drop = Vec::new();
    let mut vec2 = Vec::new();
    let mut vec3 = Vec::new();
    let mut lit = Vec::new();
    let mut processing = Vec::new();
    let mut colours = HashMap::new();

    loop {
        let leds = controller.leds_mut(0);
        if processing.len() < 20 {
            let to_light_id = rng.gen_range(0, NUM_LEDS);
            if !processing.contains(&to_light_id) {
                to_light.push(Element {
                    index: to_light_id,
                    step: 0,
                });
                let col = if pastel {
                    (rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>())
                } else {
                    let to_rand = rng.gen_range(1, 8);
                    let r = if (to_rand & 0x1) != 0 {
                        rng.gen_range(50, 255)
                    } else {
                        0
                    };
                    let g = if (to_rand & 0x2) != 0 {
                        rng.gen_range(50, 255)
                    } else {
                        0
                    };
                    let b = if (to_rand & 0x4) != 0 {
                        rng.gen_range(50, 255)
                    } else {
                        0
                    };
                    (r, g, b)
                };
                processing.push(to_light_id);
                colours.insert(to_light_id, col);
            }
        }

        if lit.len() >= 10 {
            lit.shuffle(&mut rng);
            let id = lit.pop().unwrap();

            to_drop.push(Element {
                index: id,
                step: 250,
            });
        }

        for e in &to_drop {
            let e2 = Element {
                index: e.index,
                step: e.step - 25,
            };
            let col = colours[&e.index];
            let r = (((e2.step as usize * col.0 as usize) / 255) & 255)
                .try_into()
                .unwrap();
            let g = (((e2.step as usize * col.1 as usize) / 255) & 255)
                .try_into()
                .unwrap();
            let b = (((e2.step as usize * col.2 as usize) / 255) & 255)
                .try_into()
                .unwrap();
            leds[e2.index] = [r, g, b, 0];
            if e2.step != 0 {
                vec3.push(e2);
            } else {
                let idx = processing.iter().position(|z| *z == e2.index).unwrap();
                processing.remove(idx);
            }
        }

        for e in &to_light {
            let e2 = Element {
                index: e.index,
                step: e.step + 25,
            };
            let col = colours[&e.index];
            let r = (((e2.step as usize * col.0 as usize) / 255) & 255)
                .try_into()
                .unwrap();
            let g = (((e2.step as usize * col.1 as usize) / 255) & 255)
                .try_into()
                .unwrap();
            let b = (((e2.step as usize * col.2 as usize) / 255) & 255)
                .try_into()
                .unwrap();
            leds[e2.index] = [r, g, b, 0];
            if e2.step >= 250 {
                lit.push(e2.index);
            } else {
                vec2.push(e2);
            }
        }

        to_light.clear();
        to_light.append(&mut vec2);
        to_drop.clear();
        to_drop.append(&mut vec3);

        controller.render()?;
        thread::sleep(Duration::from_millis(100));
    }
}

fn demo() -> Result<(), Box<dyn std::error::Error>> {
    let mut controller = ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0,
            ChannelBuilder::new()
                .pin(18)
                .count(NUM_LEDS.try_into().unwrap())
                .strip_type(StripType::Ws2811Gbr)
                .brightness(255)
                .build(),
        )
        .build()?;

    let mut j = 0u8;

    loop {
        //bands(&mut controller, j, true, 4, 100)?;
        tracer(&mut controller, j, (255, 0, 0), (0, 255, 0))?;
        //random_col(&mut controller, false)?;
        //rainbow_explode(&mut controller)?;

        j = j.wrapping_add(1);
        if j == 250 {
            j = 0;
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();

    if let Err(e) = demo() {
        log::error!("Error: {}", e);
    }
}
