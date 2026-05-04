use crate::constants::SimulationParams;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuParticle {
    pub pos: [f32; 2],           // 8 bytes
    pub predicted_pos: [f32; 2], // 8 bytes
    pub vel: [f32; 2],           // 8 bytes
    pub force: [f32; 2],         // 8 bytes
    pub density: f32,            // 4 bytes
    pub pressure: f32,           // 4 bytes
                                 // we have 40 bytes
                                 // from specs (https://www.w3.org/TR/WGSL/#alignment-and-size) alignment of a struct is defined as
                                 // AlignOf(S) = max(AlignOfMember(S,0), max(AlignOfMember(S,1), ... , AlignOfMember(S,N)) = 8 here
                                 // SizeOf(S) = roundUp(AlignOf(S), justPastLastMember) =
                                 // ceil(justPastLastMember / AlignOf(S)) * AlignOf(S)
                                 // where justPastLastMember = OffsetOfMember(S,N) + SizeOfMember(S,N)

                                 // justPastLastMember = 36 + 4 = 40
                                 // since pressure starts at the 36th byte and is 4 bytes
                                 // 40 is divisible by 8 so roundUp(8, 40) = ceil(40/8) * 8 = 40
}

impl GpuParticle {
    pub fn spawn_particles(params: &SimulationParams, width: u32, height: u32) -> Vec<Self> {
        let cols = (params.no_particles as f32).sqrt().ceil() as u32;
        let spacing = 1.0;
        let start_x = width as f32 / 2.0 - (cols as f32 * spacing) / 2.0;
        let start_y = height as f32 / 2.0 - (cols as f32 * spacing) / 2.0;
        let mut particles = Vec::with_capacity(params.no_particles as usize);
        for i in 0..params.no_particles {
            let x = (i % cols) as f32 * spacing + start_x;
            let y = (i / cols) as f32 * spacing + start_y;

            particles.push(GpuParticle {
                pos: [x, y],
                predicted_pos: [x, y],
                vel: [0.0, 0.0],
                force: [0.0, 0.0],
                density: 0.0,
                pressure: 0.0,
            });
        }
        particles
        // let mut rng = rand::rng();

        // for _ in 0..params.no_particles {
        //     let (x, y) = (
        //         rng.random_range(0.0..=params.width),
        //         rng.random_range(0.0..=params.height),
        //     );
        //     initial_particles.push(GpuParticle {
        //         pos: [x, y],
        //         predicted_pos: [x, y],
        //         vel: [0.0, 0.0],
        //         density: params.rest_density,
        //         pressure: 0.0,
        //         force: [0.0, 0.0],
        //     });
        // }
    }
}
