# SPH solver

A SPH solver for the GPU written in Rust.

WGPU was chosen as it seems pretty versatile, and I could turn this into a WASM component and host it in a website if I wanted. It also seemed like the most mature library out there for GPU computing with Rust.

## CPU
The CPU folder contains everything one might need to run a simulation in the CPU, except for the rendering, which in the early stages of this project was handled by [Macroquad](https://macroquad.rs/), later, this was changed to run primarily in the GPU (see the [GPU section](#GPU)). 

This project started in the GPU because it was too big of a problem for me. I didn't know anything about GPUs and graphics programming, and I knew even less about CFD techniques, let alone SPH.

I would say, if you want to the raw logic, without the preparation and meticulosity of passing the data through a buffer to the GPU, the CPU folder looks much friendlier, more on the [Improvement section](#Improvements) for this.

## GPU
The GPU folder, as you might imagine, contains everything one might need to run a simulation in the GPU.

Here you will find things you won't be able to find in the CPU folder, particularly, the [context.rs](src/gpu/context.rs) file the [pipelines.rs](src/gpu/pipelines.rs) file and the [shaders](src/gpu/shaders) folder. The specific explanation for what each thing does is documented in each file.

To start, I tried using macroquad and wgpu together, but as it turns out macroquad uses its own version of wgpu which is pretty old, and so I was getting conflicts in the build. So I changed to raw winit with wgpu, for the little parameters window I used egui-winit. 

Since WGSL doesn't natively support sorting, one often resorts to writting their own sorter, in this area, people had done a better job than I could, or had the patience to, so I used [someone else's](https://github.com/KeKsBoTer/wgpu_sort). Unfortunately, wgpu evolves quickly and this library was also out of sync with the wgpu version being used, for that reason I forked and raised a PR to update their sorter the forked version can be found [here](https://github.com/mgtorloni/wgpu_sort). 

## Improvements and future work
There are several areas of improvement that this project would benefit from, some of which I might indeed do at some point. Here are some:

1. Optimisations. I am sure there are a ton of optimisations I could make to increase the number of particles that one can run here. 
2. Viscosity. The obvious next step here in terms of physical quality of the simulation would be to use the Navier–Stokes equations to simulate viscosity.
3. Boundaries. Right now there are some clear problems with the boundaries which look pretty unnatural i.e. there is a layer of particles that just stays there and gets pushed up through the sides.
6. Go 3D.
4. As it is with SPH the parameters are super finnicky, and I would like to find a way to make them more stable.
5. I would like to make the simulation more stable in relation to FPS, I tried a bunch of methods, none of them worked really well.
7. Some tests...
8. Some kind of more graphics based work to make the fluid look like a fluid.

## References
This project wouldn't have happened without Sebastian Lague's famous [Coding Adventure: Simulating Fluids](https://www.youtube.com/watch?v=rSKMYc1CQHE). Here are some papers and sources I went through throughtout this journey:
- https://matthias-research.github.io/pages/publications/sca03.pdf
- https://www.cs.cmu.edu/~scoros/cs15467-s16/lectures/11-fluids2.pdf 
- https://developer.download.nvidia.com/assets/cuda/files/particles.pdf
- https://www.vmware.com/docs/exploring-the-gpu-architecture
- https://www.w3.org/TR/WGSL/




