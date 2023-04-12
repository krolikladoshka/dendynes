use std::{path::Path, rc::Rc, cell::RefCell, borrow::{BorrowMut, Borrow}, time::SystemTime, collections::HashMap};
use graphics::{image, Transformed};
use ::image::RgbaImage;
use lazy_static::lazy_static;
use log::warn;
use piston::{WindowSettings, Event, Loop, EventLoop, EventSettings, Input, Button, Key, ButtonState};
use piston_window::{PistonWindow, Texture, TextureSettings};

use crate::dendynes::{logging::init_logger, cartridge::Cartridge, ppu::{PPU, CYCLES_TO_DRAW_SCANLINE, SCANLINES_COUNT, SCREEN_WIDTH, SCREEN_HEIGHT, PALETTE}, bus::Bus};
use crate::dendynes::bus::joypad::JoypadButtons;
use self::cpu::processor::CPU;


pub mod bus;
pub mod cpu;
pub mod memory;
pub mod cartridge;
pub mod logging;
pub mod ppu;


const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

fn clock_cpu(cpu: &mut CPU, cycles_to_run: u64) {
    let mut cycles = 0;
    let cycles_since_start = cpu.bus.cpu_cycles;
    // cpu.step_for_cycles(cycles_to_run);
    while cycles < cycles_to_run {
        let elapsed_cycles = cpu.cpu_step();
        // cycles += cpu.cpu_step();
        cycles += elapsed_cycles;
        // let elapsed_cycles = cpu.bus.cpu_cycles - cycles_since_start;
        if elapsed_cycles == 0 {
            println!("No cycles elapsed! {} {} {}", cycles_since_start, elapsed_cycles, cycles);
        }
        // cycles += elapsed_cycles as usize;
        // cycles += elapsed_cpu_cycles;
    }
    // println!("run cycles: {}; {}", cycles, cycles_to_run);
}

/*
    Q - sq1
    E - cross1
    R - triangle1
    T - circle1
    1 - start1
    share - options1
    W - up1
    S - down1
    A - left1
    D - right1
 */

// well, those keys are mapped by my external DS5 mapping tool for windows
// totally inconvinient
lazy_static! {
    pub static ref USER1_INPUT_MAP: HashMap<Key, JoypadButtons>  = {
        let mut hashmap = HashMap::new();

        hashmap.insert(Key::Q, JoypadButtons::A);
        hashmap.insert(Key::E, JoypadButtons::B);
        hashmap.insert(Key::R, JoypadButtons::A);
        hashmap.insert(Key::T, JoypadButtons::A);

        hashmap.insert(Key::D1, JoypadButtons::START);
        hashmap.insert(Key::D2, JoypadButtons::SELECT);
        hashmap.insert(Key::W, JoypadButtons::UP);
        hashmap.insert(Key::S, JoypadButtons::DOWN);
        hashmap.insert(Key::A, JoypadButtons::LEFT);
        hashmap.insert(Key::D, JoypadButtons::RIGHT);

        return hashmap;
    };

    pub static ref USER2_INPUT_MAP: HashMap<Key, JoypadButtons>  = {
        let mut hashmap = HashMap::new();

        hashmap.insert(Key::U, JoypadButtons::A);
        hashmap.insert(Key::I, JoypadButtons::B);
        hashmap.insert(Key::O, JoypadButtons::A);
        hashmap.insert(Key::P, JoypadButtons::A);

        hashmap.insert(Key::D3, JoypadButtons::START);
        hashmap.insert(Key::D4, JoypadButtons::SELECT);
        hashmap.insert(Key::Up, JoypadButtons::UP);
        hashmap.insert(Key::Down, JoypadButtons::DOWN);
        hashmap.insert(Key::Left, JoypadButtons::LEFT);
        hashmap.insert(Key::Right, JoypadButtons::RIGHT);

        return hashmap;
    };
}

fn handle_user_1_input<'a>(cpu: &'a mut CPU, input: &Input) {
    match &input {
        Input::Button(button_args) => {
            if let Button::Keyboard(key) = button_args.button {
                if let Some(joypad_button) = USER1_INPUT_MAP.get(&key) {
                    match button_args.state {
                        ButtonState::Press => {
                            cpu.bus.joypads[0].press_button(*joypad_button);
                        },
                        ButtonState::Release => {
                            cpu.bus.joypads[0].release_button(*joypad_button);
                        },
                    }
                }
            }
        },
        _ => {},
    } 
}


fn handle_user_2_input<'a>(cpu: &'a mut CPU
, input: &Input) {
    match &input {
        Input::Button(button_args) => {
            if let Button::Keyboard(key) = button_args.button {
                if let Some(joypad_button) = USER2_INPUT_MAP.get(&key) {
                    match button_args.state {
                        ButtonState::Press => {
                            cpu.bus.joypads[1].press_button(*joypad_button);
                        },
                        ButtonState::Release => {
                            cpu.bus.joypads[1].release_button(*joypad_button);
                        },
                    }
                }
            }
        },
        _ => {},
    } 
}


pub fn dendy_run() {
    init_logger().unwrap();
    
    warn!("Started app");

    let current_file = Path::new(file!());

    // let cartridge_path = "tests/roms/donkey kong.nes";
    // let cartridge_path = "tests/roms/nestest.nes";
    // let cartridge_path = "tests/roms/Super_mario_brothers.nes";
    // let cartridge_path = "tests/roms/DuckTales (USA).nes";
    // let cartridge_path = "tests/roms/Bomber_man.nes";
    let cartridge_path = "tests/roms/Contra (U).nes";
    // let cartridge_path = "tests/roms/Duck Hunt (World).nes";
    let cartridge_path = current_file.parent().unwrap().join(cartridge_path);

    let mut cartridge = Rc::new(
        RefCell::new(
            Cartridge::new(cartridge_path.as_os_str().to_str().unwrap())
        )
    );
    let mut ppu_device = PPU::new(cartridge.clone());
    
    let mut bus = Bus::new(&mut ppu_device, cartridge.clone());
    let mut cpu = CPU::new(&mut bus);

    let mut window: PistonWindow = WindowSettings::new(
        "Dendynes emulator", [WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64]
    ).exit_on_esc(true)
     .build()
     .unwrap();
    
    let mut event_settings = EventSettings::new();
    event_settings.ups = 60;
    window.set_event_settings(event_settings);

    let cpu_cycles_for_frame = CYCLES_TO_DRAW_SCANLINE * (SCANLINES_COUNT as usize);
    let cpu_cycles_for_frame = ((cpu_cycles_for_frame as f32) / 3f32).round() as usize;

    let mut image_buffer = RgbaImage::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    let mut table_image_buffer_1 = RgbaImage::new(128, 128);
    let mut table_image_buffer_2 = RgbaImage::new(128, 128);

    let mut texture_context = &mut window.create_texture_context();

    let mut texture = Texture::from_image(
        texture_context, &image_buffer, &TextureSettings::new()
    ).unwrap();

    let mut table_texture_1 = Texture::from_image(
        texture_context, &table_image_buffer_1, &TextureSettings::new()
    ).unwrap();
    let mut table_texture_2 = Texture::from_image(
        texture_context, &table_image_buffer_2, &TextureSettings::new()
    ).unwrap();


    while let Some(event) = window.next() {
        match event {
            Event::Input(input, _) => {
                handle_user_1_input(&mut cpu, &input);
                handle_user_2_input(&mut cpu, &input);
            },
            Event::Loop(kind) => {
                match kind {
                    Loop::Update(args) => {
                        // let cycles_per_update = ((cpu_cycles_for_frame as f64) * args.dt).round() as usize;
                        let cycles_per_update = (cpu_cycles_for_frame) as u64;
                        let start = SystemTime::now();
                        clock_cpu(cpu.borrow_mut(), cycles_per_update);
                        let end = SystemTime::now();

                        // println!(
                        //     "Update took: dt {}; clock: {}ms; {}s",
                        //     args.dt,
                        //     end.duration_since(start).unwrap().as_millis(),
                        //     end.duration_since(start).unwrap().as_secs_f32(),
                        // );
                    },
                    Loop::Render(_args) => {
                        window.draw_2d(&event, |c, g, d| {
                            let start = SystemTime::now();
                            let screen = &cpu.borrow().bus.ppu.screen;
                            for y in 0..SCREEN_HEIGHT {
                                for x in 0..SCREEN_WIDTH {
                                    let nes_color = screen[y][x];
                                    let pixel = PALETTE[nes_color as usize];
                                    image_buffer.put_pixel(x as u32, y as u32, ::image::Rgba(
                                        [pixel[0], pixel[1], pixel[2], 255],
                                    )
                                    );
                                }
                            }                     
                            let end1 = SystemTime::now();

                            {
                                cpu.borrow_mut().bus.ppu.draw_pattern_tables();
                            }
                            for y in 0..128 {
                                for x in 0..128 {
                                    let nes_color = cpu.borrow().bus.ppu.debug_pattern_tables[0][y][x];
                                    let pixel = PALETTE[nes_color as usize];
                                    table_image_buffer_1.put_pixel(x as u32, y as u32, ::image::Rgba(
                                        [pixel[0], pixel[1], pixel[2], 255],
                                    )
                                    );
                                }
                            } 
                            for y in 0..128 {
                                for x in 0..128 {
                                    let nes_color = cpu.borrow().bus.ppu.debug_pattern_tables[1][y][x];
                                    let pixel = PALETTE[nes_color as usize];
                                    table_image_buffer_2.put_pixel(x as u32, y as u32, ::image::Rgba(
                                        [pixel[0], pixel[1], pixel[2], 255],
                                    )
                                    );
                                }
                            }               

                            // println!("######### RENDER");
                            // println!("{:?}", cpu.borrow().bus.ppu.screen);

                            texture.update(texture_context, &image_buffer).unwrap();
                            table_texture_1.update(texture_context, &table_image_buffer_1).unwrap();
                            table_texture_2.update(texture_context, &table_image_buffer_1).unwrap();
                            
                            image(&texture, c.transform.scale(2f64, 2f64), g);
                            image(&table_texture_1, c.transform.trans((SCREEN_WIDTH * 2) as f64, 0f64), g);
                            image(&table_texture_2, c.transform.trans((SCREEN_WIDTH * 3) as f64, 0f64), g);
                            let end2 = SystemTime::now();

                            texture_context.encoder.flush(d);
                            let end3 = SystemTime::now();
                            // println!(
                            //     "Draw took: {}; Texture update: {}; Flush: {}",
                            //     end1.duration_since(start).unwrap().as_secs_f32(),
                            //     end2.duration_since(start).unwrap().as_secs_f32(),
                            //     end3.duration_since(start).unwrap().as_secs_f32()
                            // )
                        });
                    },
                    _ => {},
                }
            },
            _ => {}
        }
    }
    
}