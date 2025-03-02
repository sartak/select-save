pub mod screen;

use crate::manager::Manager;
use anyhow::{Result, anyhow};
use sdl2::event::{Event, WindowEvent};
use sdl2::image::InitFlag;
use sdl2::render::BlendMode;
use std::path::Path;
use tracing::{debug, trace};

pub enum Button {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    L1 = 4,
    R1 = 5,
    Start = 6,
    Select = 7,
    L3 = 8,
    R3 = 9,
    L2 = 10,
    R2 = 11,
    Up,
    Down,
    Left,
    Right,
}

pub enum Action {
    Continue,
    Screenshot,
    Quit,
}

pub const BUTTON_A: u8 = Button::A as u8;
pub const BUTTON_B: u8 = Button::B as u8;
pub const BUTTON_X: u8 = Button::X as u8;
pub const BUTTON_Y: u8 = Button::Y as u8;
pub const BUTTON_L1: u8 = Button::L1 as u8;
pub const BUTTON_R1: u8 = Button::R1 as u8;
pub const BUTTON_START: u8 = Button::Start as u8;
pub const BUTTON_SELECT: u8 = Button::Select as u8;
pub const BUTTON_L3: u8 = Button::L3 as u8;
pub const BUTTON_R3: u8 = Button::R3 as u8;
pub const BUTTON_L2: u8 = Button::L2 as u8;
pub const BUTTON_R2: u8 = Button::R2 as u8;

struct UI<'a, 'b> {
    sdl_context: sdl2::Sdl,
    screen: screen::Screen<'a, 'b>,
    manager: Manager,
}

pub fn run(
    screen_width: u32,
    screen_height: u32,
    font_path: &Path,
    manager: Manager,
) -> Result<()> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let controller_subsystem = sdl_context.game_controller().unwrap();
    let controllers = controller_subsystem.num_joysticks().unwrap();
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();

    if controllers == 0 {
        return Err(anyhow!("No controllers found"));
    }

    debug!(
        "Found {controllers} controllers (primary {}: {})",
        if controller_subsystem.is_game_controller(0) {
            "controller"
        } else {
            "joystick"
        },
        controller_subsystem.name_for_index(0).unwrap()
    );

    let controller = controller_subsystem.open(0).unwrap();
    debug!("Controller mapping: {}", controller.mapping());

    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem
        .window("select-save", screen_width, screen_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(BlendMode::Blend);

    let screen = screen::Screen::new(canvas, screen_width, screen_height, &ttf_context, font_path);

    let mut ui = UI {
        sdl_context,
        screen,
        manager,
    };

    ui.run()
}

impl<'a, 'b> UI<'a, 'b> {
    fn run(&mut self) -> Result<()> {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        loop {
            self.screen.clear(self.manager.background_color());
            self.manager.draw(&mut self.screen);
            self.screen.present();

            match self.input(&mut event_pump) {
                Action::Continue => {}
                Action::Quit => break,
                Action::Screenshot => {
                    self.screen.clear(self.manager.background_color());
                    self.manager.draw(&mut self.screen);
                    self.screen.take_screenshot();
                }
            }
        }

        Ok(())
    }

    fn input(&mut self, event_pump: &mut sdl2::EventPump) -> Action {
        loop {
            return match event_pump.wait_event() {
                Event::Window {
                    win_event: WindowEvent::Shown,
                    ..
                } => {
                    // force rerender
                    Action::Continue
                }

                Event::Quit { .. } => Action::Quit,

                Event::JoyButtonDown {
                    button_idx: BUTTON_A,
                    ..
                } => self.manager.pressed(Button::A),

                Event::JoyButtonDown {
                    button_idx: BUTTON_B,
                    ..
                } => self.manager.pressed(Button::B),

                Event::JoyButtonDown {
                    button_idx: BUTTON_X,
                    ..
                } => self.manager.pressed(Button::X),

                Event::JoyButtonDown {
                    button_idx: BUTTON_Y,
                    ..
                } => self.manager.pressed(Button::Y),

                Event::JoyButtonDown {
                    button_idx: BUTTON_START,
                    ..
                } => self.manager.pressed(Button::Start),

                Event::JoyButtonDown {
                    button_idx: BUTTON_SELECT,
                    ..
                } => self.manager.pressed(Button::Select),

                Event::JoyButtonDown {
                    button_idx: BUTTON_L1,
                    ..
                } => self.manager.pressed(Button::L1),

                Event::JoyButtonDown {
                    button_idx: BUTTON_L2,
                    ..
                } => self.manager.pressed(Button::L2),

                Event::JoyButtonDown {
                    button_idx: BUTTON_L3,
                    ..
                } => self.manager.pressed(Button::L3),

                Event::JoyButtonDown {
                    button_idx: BUTTON_R1,
                    ..
                } => self.manager.pressed(Button::R1),

                Event::JoyButtonDown {
                    button_idx: BUTTON_R2,
                    ..
                } => self.manager.pressed(Button::R2),

                Event::JoyButtonDown {
                    button_idx: BUTTON_R3,
                    ..
                } => self.manager.pressed(Button::R3),

                Event::ControllerButtonDown {
                    button: sdl2::controller::Button::DPadUp,
                    ..
                } => self.manager.pressed(Button::Up),

                Event::ControllerButtonDown {
                    button: sdl2::controller::Button::DPadDown,
                    ..
                } => self.manager.pressed(Button::Down),

                Event::ControllerButtonDown {
                    button: sdl2::controller::Button::DPadLeft,
                    ..
                } => self.manager.pressed(Button::Left),

                Event::ControllerButtonDown {
                    button: sdl2::controller::Button::DPadRight,
                    ..
                } => self.manager.pressed(Button::Right),

                e => {
                    trace!("Got unhandled event {e:?}");
                    continue;
                }
            };
        }
    }
}
