use crate::glam::Vec2;
use crate::vec2;
use egui_macroquad::egui;

pub const NO_PARTICLES: usize = 100;
pub const WIDTH: i32 = 1800;
pub const HEIGHT: i32 = 1000;

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
            max_vel: 40000.0,
            radius: 3.0,
            mass: 1.0,
            rest_density: 0.0050,
            gravity: vec2(0.0, 2780000.0),
            gas_constant: 5065420032.0,
            influence_radius: 25.0,
            cell_size: 25.0,
            damping: 0.3,
            mouse_influence_radius: 70.0,
            mouse_force: 13670.0,
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
                ui.add(
                    egui::DragValue::new(&mut self.radius)
                        .speed(0.1)
                        .range(0.5..=f32::MAX),
                );
                ui.end_row();

                ui.label("Mass");
                ui.add(egui::DragValue::new(&mut self.mass).speed(0.1));
                ui.end_row();

                ui.label("Rest Density");
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
