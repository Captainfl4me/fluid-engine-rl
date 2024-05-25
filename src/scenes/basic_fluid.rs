use crate::colors::*;
use crate::scenes::Scene;
use raylib::prelude::*;

pub struct BasicFuildScene {}
impl Scene for BasicFuildScene {
    fn get_title(&self) -> &str {
        "Basic 2D fluid"
    }

    fn has_background(&self) -> bool {
        true
    }

    fn help_text(&self) -> Vec<&str> {
        vec![]
    }

    fn update(&mut self, rl_handle: &mut RaylibHandle) {
        
    }

    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle) {
        
    }
}
impl Default for BasicFuildScene {
    fn default() -> Self {
        BasicFuildScene {}
    }
}
