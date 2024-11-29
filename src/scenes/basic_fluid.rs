use crate::colors::*;
use crate::scenes::Scene;
use crate::fluid_engine::*;
use raylib::prelude::*;

const GRID_SIZE: (usize, usize) = (256, 128);
const TIMESTEP: f64 = 0.1;

pub struct BasicFuildScene {
    fluid_domain: FluidDomain<{GRID_SIZE.0}, {GRID_SIZE.1}>,
    render_image: Image,
    render_texture: Texture2D,
}
impl BasicFuildScene {
    pub fn new(rl_handle: &mut RaylibHandle, rl_thread: &RaylibThread) -> Self {
        let mut image = Image::gen_image_color(GRID_SIZE.0 as i32, GRID_SIZE.1 as i32, Color::new(0, 0, 0, 0));
        let mut fluid_domain = FluidDomain::new(TIMESTEP);

        let wall_color = Color::new(0, 0, 0, 0);
        // Set static wall as boundary
        for x_id in 0..GRID_SIZE.0 {
            fluid_domain.set_cell_state(x_id, GRID_SIZE.1 - 1, CellState::Wall);
            image.draw_pixel(x_id as i32, (GRID_SIZE.1 - 1) as i32, wall_color);
        }
        for y_id in 1..(GRID_SIZE.1 - 1) {
            fluid_domain.set_cell_state(0, y_id, CellState::Wall);
            fluid_domain.set_cell_state(GRID_SIZE.0 - 1, y_id, CellState::Wall);
            image.draw_pixel(0, y_id as i32, wall_color);
            image.draw_pixel((GRID_SIZE.0 - 1) as i32, y_id as i32, wall_color);
        }

        BasicFuildScene {
            fluid_domain,
            render_texture: rl_handle
                .load_texture_from_image(rl_thread, &image)
                .unwrap(),
            render_image: image,
        }
    }
}

impl Scene for BasicFuildScene {
    fn get_title(&self) -> &str {
        "Basic 2D fluid"
    }

    fn has_background(&self) -> bool {
        false
    }

    fn help_text(&self) -> Vec<&str> {
        vec![]
    }

    fn update(&mut self, _rl_handle: &mut RaylibHandle) {
        // Updating velocity based on external force
        for line_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
            for column_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                self.fluid_domain.fluid_grid[line_id][column_id].velocity.1 -=
                    ((self.fluid_domain.fluid_grid[line_id][column_id - 1].state as u8) as f64)
                        * TIMESTEP
                        * 9.81f64;
                self.fluid_domain.fluid_grid[line_id][column_id].pressure = 0.0;
            }
        }

        self.fluid_domain.solve_grid_incompressibility();
        self.fluid_domain.apply_advection();

        let (mut min_pressure_in_grid, mut max_pressure_in_grid) = (f64::MAX, 0.0);
        for x_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                if min_pressure_in_grid > self.fluid_domain.fluid_grid[x_id][y_id].pressure {
                    min_pressure_in_grid = self.fluid_domain.fluid_grid[x_id][y_id].pressure;
                }
                if max_pressure_in_grid < self.fluid_domain.fluid_grid[x_id][y_id].pressure {
                    max_pressure_in_grid = self.fluid_domain.fluid_grid[x_id][y_id].pressure;
                }
            }
        }

        for x_id in 1..self.fluid_domain.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_domain.fluid_grid[0].len() - 1 {
                let pressure_level = (self.fluid_domain.fluid_grid[x_id][y_id].pressure - min_pressure_in_grid)
                    / (max_pressure_in_grid - min_pressure_in_grid);
                let color = if self.fluid_domain.fluid_grid[x_id][y_id].state == CellState::Wall {
                    Color::new(0, 0, 0, 255)
                } else {
                    hsl_to_rgb((1.0 - pressure_level) / 6.0, 1.0, 1.0)
                };

                self.render_image
                    .draw_pixel(x_id as i32, y_id as i32, color);

            }
        }

        // Copy data to shader
        println!("min_value: {}", min_pressure_in_grid);
        println!("max_value: {}", max_pressure_in_grid);
    }

    fn draw(&mut self, rl_handle: &mut RaylibDrawHandle) {
        let arr: Vec<u8> = self
            .render_image
            .get_image_data()
            .iter()
            .flat_map(|c| c.color_to_int().to_be_bytes())
            .collect();
        self.render_texture.update_texture(&arr);
        rl_handle.draw_texture(&self.render_texture, (rl_handle.get_screen_width() - GRID_SIZE.0 as i32) / 2, (rl_handle.get_screen_height() - GRID_SIZE.1 as i32) / 2, COLOR_WHITE);
    }
}
