#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Wall = 0,
    Fluid = 1,
}

#[derive(Copy, Clone)]
pub struct FluidCell {
    pub velocity: (f64, f64),
    pub divergence: f64,
    pub state: CellState,
    pub pressure: f64,
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

const GRID_SPACING: f64 = 1.0;
const FLUID_DENSITY: f64 = 1000f64; // Water 1000 kg/m³

pub struct FluidDomain<const GRID_SIZE_X: usize, const GRID_SIZE_Y: usize> {
    pub fluid_grid: [[FluidCell; GRID_SIZE_Y]; GRID_SIZE_X],
    pub timestep : f64,
}
impl<const GRID_SIZE_X: usize, const GRID_SIZE_Y: usize> FluidDomain<GRID_SIZE_X, GRID_SIZE_Y> {
    pub fn new(timestep: f64) -> Self {
        FluidDomain {
            fluid_grid: [[FluidCell::default(); GRID_SIZE_Y]; GRID_SIZE_X],
            timestep,
        }
    }

    pub fn set_cell_state(&mut self, x_id: usize, y_id: usize, new_state: CellState) {
        if self.fluid_grid[x_id][y_id].state == new_state {
            return;
        }

        if self.fluid_grid[x_id][y_id].state == CellState::Fluid {
            self.fluid_grid[x_id][y_id].velocity.0 = 0.0;
            self.fluid_grid[x_id][y_id].velocity.1 = 0.0;
            if y_id + 1 < GRID_SIZE_Y {
                self.fluid_grid[x_id][y_id + 1].velocity.1 = 0.0;
            }
            if x_id + 1 < GRID_SIZE_X {
                self.fluid_grid[x_id + 1][y_id].velocity.0 = 0.0;
            }
        }
        self.fluid_grid[x_id][y_id].state = new_state;
    }

    pub fn sample_grid_velocity_u(&self, x: f64, y: f64) -> f64 {
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

    pub fn sample_grid_velocity_v(&self, x: f64, y: f64) -> f64 {
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
        w00 * w10 * self.fluid_grid[x_id][y_id].velocity.1
            + w01 * w10 * self.fluid_grid[x_id + 1][y_id].velocity.1
            + w01 * w11 * self.fluid_grid[x_id + 1][y_id + 1].velocity.1
            + w00 * w11 * self.fluid_grid[x_id][y_id + 1].velocity.1
    }

    pub fn solve_grid_incompressibility(&mut self) {
        let mut first_loop = true;
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
                        * (FLUID_DENSITY * GRID_SPACING / self.timestep);

                    // let cell_velocity = (self.fluid_grid[x_id][y_id].velocity.0.powi(2)
                    //     + self.fluid_grid[x_id][y_id].velocity.1.powi(2))
                    // .sqrt() as f32;
                }
            }
            first_loop = false;
        }
    }

    pub fn apply_advection(&mut self) {
        let mut new_grid = self.fluid_grid;
        for x_id in 1..self.fluid_grid.len() - 1 {
            for y_id in 1..self.fluid_grid[0].len() - 1 {
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
                        (x_id as f64) * GRID_SPACING - self.timestep * u,
                        (y_id as f64 + 0.5) * GRID_SPACING - self.timestep * v,
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
                        (x_id as f64 + 0.5) * GRID_SPACING - self.timestep * u,
                        (y_id as f64) * GRID_SPACING - self.timestep * v,
                    );
                    new_grid[x_id][y_id].velocity.1 =
                        self.sample_grid_velocity_v(last_point.0, last_point.1);
                } else {
                    new_grid[x_id][y_id].velocity.1 = 0.0;
                }
            }
        }
        self.fluid_grid = new_grid;
    }
}
