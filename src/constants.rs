use crate::glam::Vec2;
use crate::vec2;
use egui_macroquad::egui;

pub const NO_PARTICLES: usize = 2000;
pub const MAX_VEL: f32 = 5000.0;
pub const WIDTH: i32 = 1800;
pub const HEIGHT: i32 = 1000;
// pub const SCALE: f32 = 50.0;
pub const RADIUS: f32 = 5.0;
pub const MASS: f32 = 1.0;
pub const REST_DENSITY: f32 = 0.0018;
// pub const GRAVITY: Vec2 = vec2(0.0, 0.0);
pub const GRAVITY: Vec2 = vec2(0.0, 80000.0);
pub const GAS_CONSTANT: f32 = 75000000.0;
pub const INFLUENCE_RADIUS: f32 = 40.0;
pub const CELL_SIZE: f32 = INFLUENCE_RADIUS;
pub const DAMPING: f32 = 0.8;
pub const MOUSE_INFLUENCE_RADIUS: f32 = 70.0;
pub const MOUSE_FORCE_STRENGTH: f32 = 2000.0;

pub struct SimulationParams {
    pub max_vel: f32,
    pub radius: f32,
    pub mass: f32,
    pub rest_density: f32,
    pub gravity: Vec2,
    pub gas_constant: f32,
    pub influence_radius: f32,
    pub cell_size: f32,
    pub damping: f32,
    pub mouse_influence_radius: f32,
    pub mouse_force: f32,
}
impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            max_vel: 5000.0,
            radius: 5.0,
            mass: 1.0,
            rest_density: 0.0018,
            gravity: vec2(0.0, 80000.0),
            gas_constant: 75000000.0,
            influence_radius: 40.0,
            cell_size: 40.0,
            damping: 0.8,
            mouse_influence_radius: 70.0,
            mouse_force: 2000.0,
        }
    }
}

impl SimulationParams {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("sim_params_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Max Velocity");
                ui.add(egui::DragValue::new(&mut self.max_vel).speed(10.0));
                ui.end_row();

                ui.label("Radius");
                ui.add(egui::DragValue::new(&mut self.radius).speed(0.1));
                ui.end_row();

                ui.label("Mass");
                ui.add(egui::DragValue::new(&mut self.mass).speed(0.1));
                ui.end_row();

                ui.label("Rest Density");
                // Speed is very low here to allow fine-tuning of small decimals
                ui.add(
                    egui::DragValue::new(&mut self.rest_density)
                        .speed(0.0001)
                        .max_decimals(4),
                );
                ui.end_row();

                ui.label("Gravity");
                ui.horizontal(|ui| {
                    ui.label("X");
                    ui.add(egui::DragValue::new(&mut self.gravity.x).speed(100.0));
                    ui.label("Y");
                    ui.add(egui::DragValue::new(&mut self.gravity.y).speed(100.0));
                });
                ui.end_row();

                ui.label("Gas Constant");
                // Speed is very high here to handle the millions range
                ui.add(egui::DragValue::new(&mut self.gas_constant).speed(10000.0));
                ui.end_row();

                ui.label("Influence Radius");
                ui.add(egui::DragValue::new(&mut self.influence_radius).speed(1.0));
                ui.end_row();

                ui.label("Cell Size");
                ui.add(egui::DragValue::new(&mut self.cell_size).speed(1.0));
                ui.end_row();

                ui.label("Damping");
                ui.add(
                    egui::DragValue::new(&mut self.damping)
                        .speed(0.01)
                        .range(0.0..=1.0),
                );
                ui.end_row();

                ui.heading("Mouse Interaction");
                ui.end_row();

                ui.label("Mouse Radius");
                ui.add(egui::DragValue::new(&mut self.mouse_influence_radius).speed(1.0));
                ui.end_row();

                ui.label("Mouse Force");
                ui.add(egui::DragValue::new(&mut self.mouse_force).speed(10.0));
                ui.end_row();
            });
    }
}
