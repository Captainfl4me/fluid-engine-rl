use crate::colors::*;
use crate::scenes::Scene;
use raylib::prelude::*;

const GRID_SIZE: (usize, usize) = (256, 128);
const GRID_SPACING: f64 = 1.0; // 1 m
const TIMESTEP: f64 = 0.1;
const FLUID_DENSITY: f64 = 1000f64; // Water 1000 kg/m³

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState {
    Wall = 0,
    Fluid = 1,
}

#[derive(Copy, Clone)]
struct FluidCell {
    velocity: (f64, f64),
    divergence: f64,
    state: CellState,
    pressure: f64,
}
impl Default for FluidCell {
    fn default() -> Self {
        FluidCell {
            velocity: (0.0, 0.0), // (u, v)
            divergence: 0f64,
            state: CellState::Fluid,
            pressure: 0f64,
        }
    }
}

pub struct BasicFuildScene {
    fluid_grid: [[FluidCell; GRID_SIZE.1]; GRID_SIZE.0],
    render_image: Image,
    render_texture: Texture2D,
}
impl BasicFuildScene {
    pub fn new(rl_handle: &mut RaylibHandle, rl_thread: &RaylibThread) -> Self {
        let mut grid = [[FluidCell::default(); GRID_SIZE.1]; GRID_SIZE.0];
        let mut image = Image::gen_image_color(GRID_SIZE.0 as i32, GRID_SIZE.1 as i32, COLOR_RED);

        let wall_color = Color::new(0, 0, 0, 0);
        // Set static wall as boundary
        let height = grid[0].len();
        for (x_id, column) in grid.iter_mut().enumerate() {
            //column[0].state = CellState::Wall;
            column[height - 1].state = CellState::Wall;
            image.draw_pixel(x_id as i32, 0, wall_color);
            image.draw_pixel(x_id as i32, (height - 1) as i32, wall_color);
        }
        let width = grid.len();
        for y_id in 1..height - 1 {
            grid[0][y_id].state = CellState::Wall;
            grid[width - 1][y_id].state = CellState::Wall;
            image.draw_pixel(0, y_id as i32, wall_color);
            image.draw_pixel((width - 1) as i32, y_id as i32, wall_color);
        }

        BasicFuildScene {
            fluid_grid: grid,
            render_texture: rl_handle
                .load_texture_from_image(rl_thread, &image)
                .unwrap(),
            render_image: image,
        }
    }
}
impl BasicFuildScene {
    fn sample_grid_velocity_u(&self, x: f64, y: f64) -> f64 {
        let mut x_id = (x / GRID_SPACING).floor() as i64;
        if x_id < 0 {
            x_id = 0;
        }
        let mut x_id = x_id as usize;
        if x_id >= self.fluid_grid.len() - 1 {
            x_id = self.fluid_grid.len() - 2;
        }

        let mut y_id = (y / GRID_SPACING - 0.5).floor() as i64;
        if y_id < 0 {
            y_id = 0;
        }
        let mut y_id = y_id as usize;
        if y_id >= self.fluid_grid[0].len() - 1 {
            y_id = self.fluid_grid[0].len() - 2;
        }

        // Relative position from velocity vectors
        let x_relative_pos = (x - (x_id as f64) * GRID_SPACING) / GRID_SPACING;
        let y_relative_pos = (y - (y_id as f64 + 0.5) * GRID_SPACING) / GRID_SPACING;
        let w00 = 1.0 - x_relative_pos;
        let w10 = 1.0 - y_relative_pos;
        let w01 = x_relative_pos;
        let w11 = y_relative_pos;

        // Compute u
        w00 * w10 * self.fluid_grid[x_id][y_id].velocity.0
            + w01 * w10 * self.fluid_grid[x_id + 1][y_id].velocity.0
            + w01 * w11 * self.fluid_grid[x_id + 1][y_id + 1].velocity.0
            + w00 * w11 * self.fluid_grid[x_id][y_id + 1].velocity.0
    }
    fn sample_grid_velocity_v(&self, x: f64, y: f64) -> f64 {
        let mut x_id = (x / GRID_SPACING - 0.5).floor() as i64;
        if x_id < 0 {
            x_id = 0;
        }
        let mut x_id = x_id as usize;
        if x_id >= self.fluid_grid.len() - 1 {
            x_id = self.fluid_grid.len() - 2;
        }

        let mut y_id = (y / GRID_SPACING).floor() as i64;
        if y_id < 0 {
            y_id = 0;
        }
        let mut y_id = y_id as usize;
        if y_id >= self.fluid_grid[0].len() - 1 {
            y_id = self.fluid_grid[0].len() - 2;
        }

        // Relative position from velocity vectors
        let x_relative_pos = (x - (x_id as f64 + 0.5) * GRID_SPACING) / GRID_SPACING;
        let y_relative_pos = (y - (y_id as f64) * GRID_SPACING) / GRID_SPACING;
        let w00 = 1.0 - x_relative_pos;
        let w10 = 1.0 - y_relative_pos;
        let w01 = x_relative_pos;
        let w11 = y_relative_pos;

        // Compute v
        w00 * w10 * self.fluid_grid[x_id][y_id].velocity.0
            + w01 * w10 * self.fluid_grid[x_id + 1][y_id].velocity.0
            + w01 * w11 * self.fluid_grid[x_id + 1][y_id + 1].velocity.0
            + w00 * w11 * self.fluid_grid[x_id][y_id + 1].velocity.0
    }

    fn solve_grid_incompressibility(&mut self) -> (f64, f64) {
        let mut first_loop = true;
        let mut min_pressure_in_grid = f64::MAX;
        let mut max_pressure_in_grid = 0f64;
        // Resolve fluid grid (Compute divergence and force incompressibility)
        for _ in 0..40 {
            for x_id in 1..self.fluid_grid.len() - 1 {
                for y_id in 1..self.fluid_grid[0].len() - 1 {
                    if self.fluid_grid[x_id][y_id].state == CellState::Wall {
                        continue;
                    }

                    let number_of_fluid_cell = ((self.fluid_grid[x_id + 1][y_id].state as u8)
                        + (self.fluid_grid[x_id - 1][y_id].state as u8)
                        + (self.fluid_grid[x_id][y_id + 1].state as u8)
                        + (self.fluid_grid[x_id][y_id - 1].state as u8))
                        as f64;
                    if number_of_fluid_cell == 0.0 {
                        continue;
                    }

                    let mut divergence = self.fluid_grid[x_id][y_id].velocity.0
                        - self.fluid_grid[x_id + 1][y_id].velocity.0
                        - self.fluid_grid[x_id][y_id + 1].velocity.1
                        + self.fluid_grid[x_id][y_id].velocity.1;
                    if first_loop {
                        self.fluid_grid[x_id][y_id].divergence = divergence;
                    }
                    divergence *= 1.9;

                    self.fluid_grid[x_id][y_id].velocity.1 -=
                        ((self.fluid_grid[x_id][y_id - 1].state as u8) as f64) * divergence
                            / number_of_fluid_cell;
                    self.fluid_grid[x_id][y_id + 1].velocity.1 +=
                        ((self.fluid_grid[x_id][y_id + 1].state as u8) as f64) * divergence
                            / number_of_fluid_cell;
                    self.fluid_grid[x_id][y_id].velocity.0 -=
                        ((self.fluid_grid[x_id - 1][y_id].state as u8) as f64) * divergence
                            / number_of_fluid_cell;
                    self.fluid_grid[x_id + 1][y_id].velocity.0 +=
                        ((self.fluid_grid[x_id + 1][y_id].state as u8) as f64) * divergence
                            / number_of_fluid_cell;

                    self.fluid_grid[x_id][y_id].pressure -= (divergence / number_of_fluid_cell)
                        * (FLUID_DENSITY * GRID_SPACING / TIMESTEP);

                    // let cell_velocity = (self.fluid_grid[x_id][y_id].velocity.0.powi(2)
                    //     + self.fluid_grid[x_id][y_id].velocity.1.powi(2))
                    // .sqrt() as f32;
                    let cell_pressure = self.fluid_grid[x_id][y_id].pressure;

                    if min_pressure_in_grid > cell_pressure {
                        min_pressure_in_grid = cell_pressure;
                    }
                    if max_pressure_in_grid < cell_pressure {
                        max_pressure_in_grid = cell_pressure;
                    }
                }
            }
            first_loop = false;
        }

        (min_pressure_in_grid, max_pressure_in_grid)
    }
}
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
        // Updating velocity based on external force
        for line_id in 1..self.fluid_grid.len() - 1 {
            for column_id in 1..self.fluid_grid[0].len() - 1 {
                self.fluid_grid[line_id][column_id].velocity.1 -=
                    ((self.fluid_grid[line_id][column_id - 1].state as u8) as f64)
                        * TIMESTEP
                        * 9.81f64;
                self.fluid_grid[line_id][column_id].pressure = 0.0;
            }
        }

        let (min_pressure_in_grid, max_pressure_in_grid) = self.solve_grid_incompressibility();

        // Compute advection
        let mut new_grid = self.fluid_grid;
        for x_id in 1..self.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_grid[0].len() - 1 {
                let pressure_level = (self.fluid_grid[x_id][y_id].pressure - min_pressure_in_grid)
                    / (max_pressure_in_grid - min_pressure_in_grid);
                let color = if self.fluid_grid[x_id][y_id].state == CellState::Wall {
                    Color::new(0, 0, 0, 0)
                } else {
                    Color::new((pressure_level * 255.0) as u8, 0, 0, 255)
                };
                self.render_image
                    .draw_pixel(x_id as i32, y_id as i32, color);

                if self.fluid_grid[x_id][y_id].state != CellState::Wall
                    && self.fluid_grid[x_id - 1][y_id].state != CellState::Wall
                {
                    // Compute for U point
                    let u = self.fluid_grid[x_id][y_id].velocity.0;
                    let v = (self.fluid_grid[x_id][y_id].velocity.1
                        + self.fluid_grid[x_id][y_id + 1].velocity.1
                        + self.fluid_grid[x_id - 1][y_id].velocity.1
                        + self.fluid_grid[x_id - 1][y_id + 1].velocity.1)
                        / 4.0;
                    let last_point = (
                        (x_id as f64) * GRID_SPACING - TIMESTEP * v,
                        (y_id as f64) * GRID_SPACING - TIMESTEP * u,
                    );
                    new_grid[x_id][y_id].velocity.0 =
                        self.sample_grid_velocity_u(last_point.0, last_point.1);
                } else {
                    new_grid[x_id][y_id].velocity.0 = 0.0;
                }

                if self.fluid_grid[x_id][y_id].state != CellState::Wall
                    && self.fluid_grid[x_id][y_id - 1].state != CellState::Wall
                {
                    // Compute for V point
                    let u = (self.fluid_grid[x_id][y_id].velocity.0
                        + self.fluid_grid[x_id + 1][y_id].velocity.0
                        + self.fluid_grid[x_id][y_id - 1].velocity.0
                        + self.fluid_grid[x_id + 1][y_id - 1].velocity.0)
                        / 4.0;
                    let v = self.fluid_grid[x_id][y_id].velocity.1;
                    let last_point = (
                        (x_id as f64) * GRID_SPACING - TIMESTEP * v,
                        (y_id as f64) * GRID_SPACING - TIMESTEP * u,
                    );
                    new_grid[x_id][y_id].velocity.1 =
                        self.sample_grid_velocity_v(last_point.0, last_point.1);
                } else {
                    new_grid[x_id][y_id].velocity.1 = 0.0;
                }
            }
        }
        self.fluid_grid = new_grid;

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
        rl_handle.draw_texture(&self.render_texture, 0, 0, COLOR_LIGHT);
    }
}
