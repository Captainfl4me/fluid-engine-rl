use raylib::RaylibHandle;
use raylib::drawing::RaylibDrawHandle;

mod basic_fluid;
pub use basic_fluid::BasicFuildScene;

pub trait Scene {
    fn get_title(&self) -> &str;
    fn has_background(&self) -> bool;
    fn help_text(&self) -> Vec<&str>;
    /// Update the scene (only logic)
    fn update(&mut self, rl_handle: &mut RaylibHandle);
    /// Draw one frame of the scene
    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle);
}
