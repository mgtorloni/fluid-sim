use bytemuck::{Pod, Zeroable};

pub const NO_PARTICLES: u32 = 20000;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SimulationParams {
    pub width: f32,            //offset 0
    pub height: f32,           //offset 4
    pub no_particles: u32,     //offset 8
    pub max_vel: f32,          //offset 12
    pub radius: f32,           //offset 16
    pub mass: f32,             //offset 20
    pub rest_density: f32,     //offset 24
    pub dt: f32,               //offset 28
    pub gravity: [f32; 2],     //offset 32
    pub gas_constant: f32,     //offset 40
    pub influence_radius: f32, //offset 44
    pub cell_size: f32,        //offset 48
    pub damping: f32,          //offset 52
    pub _padding: [f32; 2],    // offset 56
                               // https://www.w3.org/TR/WGSL/#address-space-layout-constraints
                               // because this is going to be a uniform buffer
                               // i.e. roundUp(16, AlignOf(S))
                               // AlignOf(S) = max(AlignOfMember(S,0), max(AlignOfMember(S,1), ... , AlignOfMember(S,N)) = 8 here
                               // so roundUp(16,8) = 16  meaning that the struct is aligned to 16 bytes
                               // 64 bytes in total which is divisible by 16 so we are okay

                               // pub mouse_influence_radius: f32,
                               // pub mouse_force: f32,
}
impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            width: 1920.0,
            height: 1080.0,
            no_particles: NO_PARTICLES,
            max_vel: 40000.0,
            radius: 2.0,
            mass: 1.0,
            rest_density: 0.0050,
            dt: 1.0 / 60.0,
            gravity: [0.0, 27.0],
            gas_constant: 5065420032.0,
            influence_radius: 6.0,
            cell_size: 6.0,
            damping: 0.3,
            _padding: [0.0; 2],
            // mouse_influence_radius: 70.0,
            // mouse_force: 13670.0,
        }
    }
}
//
// impl SimulationParams {
//     pub fn ui(&mut self, ui: &mut egui::Ui) {
//         egui::Grid::new("sim_params_grid")
//             .num_columns(2)
//             .spacing([40.0, 4.0])
//             .striped(true)
//             .show(ui, |ui| {
//                 ui.label("Max Velocity");
//                 ui.add(egui::DragValue::new(&mut self.max_vel).speed(10.0));
//                 ui.end_row();
//
//                 ui.label("Radius");
//                 ui.add(
//                     egui::DragValue::new(&mut self.radius)
//                         .speed(0.1)
//                         .range(0.5..=f32::MAX),
//                 );
//                 ui.end_row();
//
//                 ui.label("Mass");
//                 ui.add(egui::DragValue::new(&mut self.mass).speed(0.1));
//                 ui.end_row();
//
//                 ui.label("Rest Density");
//                 ui.add(
//                     egui::DragValue::new(&mut self.rest_density)
//                         .speed(0.0001)
//                         .max_decimals(4),
//                 );
//                 ui.end_row();
//
//                 ui.label("Gravity");
//                 ui.horizontal(|ui| {
//                     ui.label("X");
//                     ui.add(egui::DragValue::new(&mut self.gravity.x).speed(100.0));
//                     ui.label("Y");
//                     ui.add(egui::DragValue::new(&mut self.gravity.y).speed(100.0));
//                 });
//                 ui.end_row();
//
//                 ui.label("Gas Constant");
//                 ui.add(egui::DragValue::new(&mut self.gas_constant).speed(10000.0));
//                 ui.end_row();
//
//                 ui.label("Influence Radius");
//                 ui.add(egui::DragValue::new(&mut self.influence_radius).speed(1.0));
//                 ui.end_row();
//
//                 ui.label("Cell Size");
//                 ui.add(egui::DragValue::new(&mut self.cell_size).speed(1.0));
//                 ui.end_row();
//
//                 ui.label("Damping");
//                 ui.add(
//                     egui::DragValue::new(&mut self.damping)
//                         .speed(0.01)
//                         .range(0.0..=1.0),
//                 );
//                 ui.end_row();
//
//                 ui.heading("Mouse Interaction");
//                 ui.end_row();
//
//                 ui.label("Mouse Radius");
//                 ui.add(egui::DragValue::new(&mut self.mouse_influence_radius).speed(1.0));
//                 ui.end_row();
//
//                 ui.label("Mouse Force");
//                 ui.add(egui::DragValue::new(&mut self.mouse_force).speed(10.0));
//                 ui.end_row();
//             });
//     }
// }
