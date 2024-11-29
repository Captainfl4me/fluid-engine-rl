use crate::colors::*;
use crate::fluid_engine::*;
use crate::scenes::Scene;
use raylib::prelude::*;
use std::ffi::CStr;

const GRID_SIZE: (usize, usize) = (256, 128);
const TIMESTEP: f64 = 0.01;

#[derive(Clone, Copy, Debug)]
enum ValueToDisplay {
    VelocityX,
    VelocityY,
    Pressure,
}

pub struct AdvectionFuildScene {
    fluid_domain: FluidDomain<{ GRID_SIZE.0 }, { GRID_SIZE.1 }>,
    render_image: Image,
    render_texture: Texture2D,
    dropdown_select: i32,
    dropdown_edit_mode: bool,
    value_to_display: ValueToDisplay,
    send_vel: bool,
}
impl AdvectionFuildScene {
    pub fn new(rl_handle: &mut RaylibHandle, rl_thread: &RaylibThread) -> Self {
        let mut image = Image::gen_image_color(
            GRID_SIZE.0 as i32,
            GRID_SIZE.1 as i32,
            Color::new(0, 0, 0, 255),
        );
        let mut fluid_domain = FluidDomain::new(TIMESTEP);

        let wall_color = Color::new(0, 0, 0, 0);
        // Set static wall as boundary
        for x_id in 0..GRID_SIZE.0 {
            fluid_domain.set_cell_state(x_id, GRID_SIZE.1 - 1, CellState::Wall);
            fluid_domain.set_cell_state(x_id, 0, CellState::Wall);
            image.draw_pixel(x_id as i32, (GRID_SIZE.1 - 1) as i32, wall_color);
            image.draw_pixel(x_id as i32, 0, wall_color);
        }

        AdvectionFuildScene {
            fluid_domain,
            render_texture: rl_handle
                .load_texture_from_image(rl_thread, &image)
                .unwrap(),
            render_image: image,
            dropdown_select: 0,
            dropdown_edit_mode: false,
            value_to_display: ValueToDisplay::VelocityX,
            send_vel: true,
        }
    }

    fn reset_fuild(&mut self) {
        self.fluid_domain = FluidDomain::new(TIMESTEP);
        for x_id in 0..GRID_SIZE.0 {
            self.fluid_domain
                .set_cell_state(x_id, GRID_SIZE.1 - 1, CellState::Wall);
            self.fluid_domain.set_cell_state(x_id, 0, CellState::Wall);
        }

        for x_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                self.render_image
                    .draw_pixel(x_id as i32, y_id as i32, Color::new(0, 0, 0, 255));
            }
        }
    }

    fn update_image_to_draw(&mut self, value_to_display: ValueToDisplay) {
        let (mut min_display, mut max_display) = (f64::MAX, 0.0);
        for x_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                let val = match value_to_display {
                    ValueToDisplay::VelocityX => {
                        self.fluid_domain.fluid_grid[x_id][y_id].velocity.0
                    }
                    ValueToDisplay::VelocityY => {
                        self.fluid_domain.fluid_grid[x_id][y_id].velocity.1
                    }
                    ValueToDisplay::Pressure => self.fluid_domain.fluid_grid[x_id][y_id].pressure,
                };

                if min_display > val {
                    min_display = val;
                }
                if max_display < val {
                    max_display = val;
                }
            }
        }

        for x_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                let val = match value_to_display {
                    ValueToDisplay::VelocityX => {
                        self.fluid_domain.fluid_grid[x_id][y_id].velocity.0
                    }
                    ValueToDisplay::VelocityY => {
                        self.fluid_domain.fluid_grid[x_id][y_id].velocity.1
                    }
                    ValueToDisplay::Pressure => self.fluid_domain.fluid_grid[x_id][y_id].pressure,
                };
                let display_level = (val - min_display) / (max_display - min_display);
                let color = if self.fluid_domain.fluid_grid[x_id][y_id].state == CellState::Wall {
                    Color::new(0, 0, 0, 255)
                } else {
                    hsl_to_rgb((1.0 - display_level) / 6.0, 1.0, 1.0)
                };

                self.render_image
                    .draw_pixel(x_id as i32, y_id as i32, color);
            }
        }

        // Copy data to shader
        println!("value_to_display: {:?}", value_to_display);
        println!("min_value: {}", min_display);
        println!("max_value: {}", max_display);
    }
}

const BYPASS: bool = true;

impl Scene for AdvectionFuildScene {
    fn get_title(&self) -> &str {
        "Advection test"
    }

    fn has_background(&self) -> bool {
        false
    }

    fn help_text(&self) -> Vec<&str> {
        vec![]
    }

    fn update(&mut self, rl_handle: &mut RaylibHandle) {
        if rl_handle.is_key_pressed(KeyboardKey::KEY_R) {
            self.reset_fuild();
        }
        if rl_handle.is_key_pressed(KeyboardKey::KEY_V) {
            self.send_vel = !self.send_vel;
        }

        if rl_handle.is_key_pressed(KeyboardKey::KEY_SPACE) || BYPASS {
            if self.send_vel {
                for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                    self.fluid_domain.fluid_grid[0][y_id].velocity.0 = 10f64;
                    self.fluid_domain.fluid_grid[self.fluid_domain.fluid_grid.len() - 1][y_id]
                        .velocity
                        .0 = 10f64;
                }
            } else {
                for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                    self.fluid_domain.fluid_grid[0][y_id].velocity.0 = 0f64;
                    self.fluid_domain.fluid_grid[self.fluid_domain.fluid_grid.len() - 1][y_id]
                        .velocity
                        .0 = 0f64;
                }
            }

            for x_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
                for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                    self.fluid_domain.fluid_grid[x_id][y_id].pressure = 0.0;
                }
            }

            self.fluid_domain.solve_grid_incompressibility();
            self.fluid_domain.apply_advection();
        }
        self.update_image_to_draw(self.value_to_display);
    }

    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle) {
        let arr: Vec<u8> = self
            .render_image
            .get_image_data()
            .iter()
            .flat_map(|c| c.color_to_int().to_be_bytes())
            .collect();
        self.render_texture.update_texture(&arr);
        rl_handle.draw_texture(
            &self.render_texture,
            (rl_handle.get_screen_width() - GRID_SIZE.0 as i32) / 2,
            (rl_handle.get_screen_height() - GRID_SIZE.1 as i32) / 2,
            COLOR_WHITE,
        );

        if rl_handle.gui_dropdown_box(
            Rectangle::new(10.0, 10.0, 150.0, 30.0),
            Some(CStr::from_bytes_with_nul(b"Velocity X;Velocity Y;Pressure\0").unwrap()),
            &mut self.dropdown_select,
            self.dropdown_edit_mode,
        ) {
            self.dropdown_edit_mode = !self.dropdown_edit_mode;
        }
        self.value_to_display = match self.dropdown_select {
            0 => ValueToDisplay::VelocityX,
            1 => ValueToDisplay::VelocityY,
            2 => ValueToDisplay::Pressure,
            _ => ValueToDisplay::VelocityX,
        };
    }
}
