pub mod assets;
pub mod draw;
pub mod sprite;
pub mod update;

use crate::{gui::GuiController, impfile};
use assets::models::ModelManager;
use assets::shaders::ShaderManager;
use assets::textures::TextureManager;
use egui_gl_glfw::egui::FontDefinitions;
use glfw::{GlfwReceiver, WindowEvent};
use sprite::{Asteroid, Explosion};

const DEFAULT_SPAWN_INTERVAL: f32 = 16.0;

//Application config values, these are not meant to be changed by normal users
#[derive(Default)]
struct Config {
    font_path: String,
}

pub struct Game {
    pub textures: TextureManager,
    pub shaders: ShaderManager,
    pub models: ModelManager,
    pub fonts: FontDefinitions,
    cfg: Config,
    window_w: i32,
    window_h: i32,
    //Sprites
    asteroid_spawn_timer: f32,
    spawn_interval: f32,
    pub asteroids: Vec<Asteroid>,
    pub explosions: Vec<Explosion>,
    time: f32,
    pub answer: String,
}

type EventHandler = GlfwReceiver<(f64, WindowEvent)>;

fn handle_window_resize(gamestate: &mut Game, w: i32, h: i32) {
    unsafe {
        gl::Viewport(0, 0, w, h);
        gamestate.window_w = w;
        gamestate.window_h = h;
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            textures: TextureManager::new(),
            shaders: ShaderManager::new(),
            models: ModelManager::new(),
            fonts: FontDefinitions::default(),
            cfg: Config::default(),
            window_w: 0,
            window_h: 0,
            asteroid_spawn_timer: 0.0,
            spawn_interval: DEFAULT_SPAWN_INTERVAL,
            asteroids: vec![],
            explosions: vec![],
            time: 0.0,
            answer: String::new(),
        }
    }

    pub fn load_config(&mut self, path: &str) {
        let entries = impfile::parse_file(path);
        if entries.is_empty() {
            eprintln!("Error: empty config file");
            return;
        }
        let e = &entries[0];
        self.cfg.font_path = e.get_var("font_path");
    }

    pub fn process_events(&mut self, events: &EventHandler, gui_controller: &mut GuiController) {
        for (_, event) in glfw::flush_messages(events) {
            match event {
                WindowEvent::Size(w, h) => handle_window_resize(self, w, h),
                WindowEvent::Key(glfw::Key::Enter, _, glfw::Action::Press, _) => {
                    //Clear answer
                    self.submit_answer();
                    continue;
                }
                _ => {}
            }
            gui_controller.handle_window_event(event);
        }
    }

    pub fn submit_answer(&mut self) {
        //TODO: get asteroids that have the same string as an 'answer' as the
        //string that was submitted
        self.answer.clear();
    }

    pub fn init_window_dimensions(&mut self, dimensions: (i32, i32)) {
        let (w, h) = dimensions;
        self.window_w = w;
        self.window_h = h;
    }

    pub fn time(&self) -> f32 {
        self.time
    }
}
