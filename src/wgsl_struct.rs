#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, /*Default,*/ bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformParams {
    // 0    8
    pub x_bounds: [f32; 2],
    // 8    8
    pub y_bounds: [f32; 2],
    // 16   4
    pub max_iterations: u32,
    // 20
    pub padding0: u32,
    // 24   8
    pub c: [f32; 2],
    // 32   16
    pub palette: [[f32; 4]; crate::COLOR_NUM],
}