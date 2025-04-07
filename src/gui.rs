use crate::game::Game;
use crate::game::draw::{CANVAS_H, CANVAS_W, caclulate_canv_offset, calculate_screen_scale};
use cgmath::Vector4;
use egui_backend::egui::{self, RichText};
use egui_backend::egui::{Align2, Color32, FontId, Pos2, RawInput, Rect, Ui, vec2};
use egui_backend::{EguiInputState, Painter};
use egui_gl_glfw as egui_backend;
use glfw::{Window, WindowEvent};

//gui action
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GuiAction {
    Restart,
}

//Initialized the egui input state
pub fn init_egui_input_state(window: &Window) -> EguiInputState {
    let (w, h) = window.get_size();
    let native_pixels_per_point = window.get_content_scale().0;
    let dimensions = vec2(w as f32, h as f32) / native_pixels_per_point;
    let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), dimensions);
    let raw_input = RawInput {
        screen_rect: Some(rect),
        ..Default::default()
    };
    EguiInputState::new(raw_input, native_pixels_per_point)
}

//Sets the OpenGL state for rendering gui components
pub fn set_ui_gl_state() {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ClearColor(0.4, 0.8, 1.0, 1.0);
    }
}

pub struct GuiController {
    painter: Painter,
    ctx: egui::Context,
    input_state: EguiInputState,
}

pub fn world_to_eguipos(x: f32, y: f32, w: i32, h: i32) -> Pos2 {
    let pos = Vector4::new(x / CANVAS_W, y / CANVAS_H, 0.0, 1.0);
    let canvas_pos = pos + Vector4::new(0.5, 0.5, 0.0, 0.0);

    let screen_scale = calculate_screen_scale(w, h);
    //Calculate the text position on the screen
    let (dx, dy) = caclulate_canv_offset(w, h);
    let tx = CANVAS_W * canvas_pos.x + dx / screen_scale;
    let ty = CANVAS_H * (1.0 - canvas_pos.y) + dy / screen_scale;
    Pos2::new(tx, ty)
}

fn display_asteroid_text(gamestate: &Game, ui: &Ui, w: i32, h: i32) {
    let painter = ui.painter();
    let font_id = FontId::new(16.0, egui::FontFamily::Monospace);
    for asteroid in &gamestate.asteroids {
        let text_pos = world_to_eguipos(asteroid.sprite.x, asteroid.sprite.y, w, h);
        //Display the text
        painter.text(
            text_pos,
            Align2::CENTER_CENTER,
            &asteroid.flashcard.question,
            font_id.clone(),
            Color32::WHITE,
        );
    }
}

pub fn gui_pos(x: f32, y: f32, w: i32, h: i32) -> Pos2 {
    let screen_scale = calculate_screen_scale(w, h);
    let corner = vec2(-w as f32 / 2.0, -h as f32 / 2.0) / screen_scale;
    world_to_eguipos(x, y, w, h) + corner
}

fn display_hud(gamestate: &Game, ui: &Ui, w: i32, h: i32) {
    let painter = ui.painter();
    let font_id = FontId::new(16.0, egui::FontFamily::Monospace);

    //Display health
    painter.text(
        gui_pos(40.0, -16.0, w, h),
        Align2::LEFT_TOP,
        format!("{}", gamestate.health),
        font_id.clone(),
        Color32::WHITE,
    );
    //Display score
    painter.text(
        gui_pos(16.0, -40.0, w, h),
        Align2::LEFT_TOP,
        format!("SCORE: {}", gamestate.score),
        font_id.clone(),
        Color32::WHITE,
    );
    //Display level
    painter.text(
        gui_pos(16.0, -64.0, w, h),
        Align2::LEFT_TOP,
        format!("LEVEL: {}", gamestate.level),
        font_id.clone(),
        Color32::WHITE,
    );
}

fn smoothstep_up(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(2)
}

fn display_levelup(gamestate: &Game, ui: &Ui, w: i32, h: i32) {
    if gamestate.levelup_animation_perc() <= 0.0 {
        return;
    }

    let painter = ui.painter();
    let font_id = FontId::new(64.0, egui::FontFamily::Monospace);
    let perc = gamestate.levelup_animation_perc();
    let y = if perc < 0.25 {
        -CANVAS_H - 80.0 + (80.0 + CANVAS_H / 2.0) * smoothstep_up(perc / 0.25)
    } else if (0.25..=0.75).contains(&perc) {
        -CANVAS_H / 2.0
    } else {
        -CANVAS_H / 2.0 + (80.0 + CANVAS_H / 2.0) * ((perc - 0.75) / 0.25).powi(2)
    };
    painter.text(
        gui_pos(CANVAS_W / 2.0, y, w, h),
        Align2::CENTER_CENTER,
        "LEVEL UP",
        font_id.clone(),
        Color32::WHITE,
    );
}

fn display_log(gamestate: &Game, ui: &Ui, w: i32, h: i32, pixels_per_point: f32) {
    if gamestate.log.is_empty() {
        return;
    }

    let painter = ui.painter();
    let font_id = FontId::new(16.0, egui::FontFamily::Monospace);
    for (i, log_item) in gamestate.log.iter().enumerate() {
        //Calculate gui x position
        let gui_position = gui_pos(32.0, 0.0, w, h);
        //Calculate the y position (subtract size of window at bottom of screen)
        let y = h as f32 / pixels_per_point - 56.0 - i as f32 * 24.0;
        painter.text(
            Pos2::new(gui_position.x, y),
            Align2::LEFT_BOTTOM,
            log_item.message(),
            font_id.clone(),
            Color32::from_rgb(255, 64, 64),
        );
    }
}

impl GuiController {
    pub fn init(window: &Window) -> Self {
        Self {
            painter: Painter::new(window),
            ctx: egui::Context::default(),
            input_state: init_egui_input_state(window),
        }
    }

    pub fn init_font(&self, gamestate: &Game) {
        self.ctx.set_fonts(gamestate.get_font());
    }

    pub fn handle_window_event(&mut self, event: WindowEvent) {
        egui_backend::handle_event(event, &mut self.input_state);
    }

    pub fn update_state(&mut self, w: i32, h: i32, time: f32, pixels_per_point: f32) {
        self.painter.set_size(w as u32, h as u32);
        self.input_state.input.time = Some(time as f64);
        let screen_scale = calculate_screen_scale(w, h);
        self.input_state.pixels_per_point = pixels_per_point * screen_scale;
    }

    //Display and update game gui
    pub fn display_game_gui(&mut self, gamestate: &mut Game, w: i32, h: i32) -> Option<GuiAction> {
        let mut action = None;

        let pixels_per_point = self.input_state.pixels_per_point;
        if self.ctx.pixels_per_point() != pixels_per_point {
            self.ctx.set_pixels_per_point(pixels_per_point);
        }
        self.ctx.begin_pass(self.input_state.input.take());

        //Display asteroid textures
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&self.ctx, |ui| {
                display_asteroid_text(gamestate, ui, w, h);
                display_hud(gamestate, ui, w, h);
                display_levelup(gamestate, ui, w, h);
                display_log(gamestate, ui, w, h, pixels_per_point);
            });

        //Answer input box
        egui::Window::new("bottom_panel")
            .movable(false)
            .title_bar(false)
            .scroll(true)
            .fixed_size(vec2(w as f32 / pixels_per_point - 64.0, 64.0))
            .fixed_pos(Pos2::new(24.0, h as f32 / pixels_per_point - 50.0))
            .show(&self.ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("Type your answer here (press enter to submit):");
                    ui.text_edit_singleline(&mut gamestate.answer);
                })
            });

        //Display game over screen
        if gamestate.game_over() {
            let width = w as f32 / pixels_per_point;
            let height = h as f32 / pixels_per_point;
            egui::Window::new("game_over_screen")
                .frame(egui::Frame::none().fill(Color32::from_rgba_unmultiplied(255, 0, 0, 128)))
                .movable(false)
                .title_bar(false)
                .scroll(true)
                .fixed_size(vec2(width, height))
                .fixed_pos(Pos2::new(0.0, 0.0))
                .show(&self.ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(height / 4.0);
                        ui.label(RichText::new("Game Over!").size(64.0).color(Color32::WHITE));
                        let final_score = format!("Final Score: {}", gamestate.score);
                        ui.label(RichText::new(final_score).size(16.0).color(Color32::WHITE));
                        let final_level = format!("Final Level: {}", gamestate.level);
                        ui.label(RichText::new(final_level).size(16.0).color(Color32::WHITE));
                        ui.add_space(height / 32.0);
                        let button_text = RichText::new("  Restart  ")
                            .size(20.0)
                            .color(Color32::WHITE);
                        let button = ui.button(button_text);
                        if button.clicked() {
                            //Restart session
                            action = Some(GuiAction::Restart);
                        }
                        let button_text = RichText::new(" Main Menu ")
                            .size(20.0)
                            .color(Color32::WHITE);
                        let button = ui.button(button_text);
                        if button.clicked() {
                            //Go to main menu
                            //TODO implement main menu
                            eprintln!("Main menu.");
                        }
                    })
                });
        }

        //End frame
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point: _,
            viewport_output: _,
        } = self.ctx.end_pass();

        //Handle copy pasting
        if !platform_output.copied_text.is_empty() {
            egui_backend::copy_to_clipboard(&mut self.input_state, platform_output.copied_text);
        }

        //Display
        let clipped_shapes = self.ctx.tessellate(shapes, pixels_per_point);
        self.painter
            .paint_and_update_textures(pixels_per_point, &clipped_shapes, &textures_delta);

        action
    }
}

pub fn handle_gui_action(gamestate: &mut Game, action: GuiAction) {
    match action {
        GuiAction::Restart => gamestate.restart(),
    }
}
