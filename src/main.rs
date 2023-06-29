mod mandelbrot;
mod julia;
mod wgsl_struct;

use std::collections::HashMap;
use std::sync::Arc;
use colorgrad::Gradient;
use eframe::{App, CreationContext, egui::{self, Context, plot::{Legend, PlotBounds, PlotImage}}, egui_wgpu::WgpuConfiguration, emath::Vec2, epaint::{self}, Frame, wgpu};
use crate::julia::JuliaRenderUtils;
use crate::mandelbrot::MandelbrotRenderUtils;
use crate::wgsl_struct::Vertex;

const COLOR_NUM: usize = 128;
const KEYS: [i32; 39] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38];
// static mut SELECTED: i32 =1;
const MAX_ITERATIONS: u32 = 65536;

pub struct MyApp {
    show_cpu: bool,
    show_gpu: bool,
    dirty: bool,
    mandelbrot_texture_id: epaint::TextureId,
    julia_texture_id: epaint::TextureId,
    mandelbrot_points: Arc<Vec<Vertex>>,
    julia_points: Arc<Vec<Vertex>>,
    last_selected: i32,
    selected: i32,
    text_map: HashMap<i32, String>,
    // gradient_map: HashMap<String, dyn Fn() -> Gradient>,
    gradient_map: HashMap<i32, fn() -> Gradient>,
    max_iterations: u32,
    show_mandelbrot: bool,
    show_julia: bool,
    c: [f32; 2],
}

impl MyApp {
    pub fn new<'a>(cc: &'a CreationContext<'a>, palette: [[f32; 4]; COLOR_NUM]) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;
        let target_format = wgpu_render_state.target_format;

        // let target_format = wgpu::TextureFormat::Rgba32Float;

        let mandelbrot_util = MandelbrotRenderUtils::new(device, target_format, palette, MAX_ITERATIONS);
        let mandelbrot_texture_id = {
            let mut renderer = wgpu_render_state.renderer.write();
            renderer.register_native_texture(device, &mandelbrot_util.create_view(), wgpu::FilterMode::Linear)
        };

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(mandelbrot_util);

        let julia_util = julia::JuliaRenderUtils::new(device, target_format, palette, MAX_ITERATIONS);
        let julia_texture_id = {
            let mut renderer = wgpu_render_state.renderer.write();
            renderer.register_native_texture(device, &julia_util.create_view(), wgpu::FilterMode::Linear)
        };
        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(julia_util);

        let presets = [colorgrad::cubehelix_default, colorgrad::inferno, colorgrad::magma, colorgrad::turbo, colorgrad::cividis, colorgrad::sinebow, colorgrad::rainbow, colorgrad::warm, colorgrad::cool, colorgrad::plasma, colorgrad::viridis, colorgrad::spectral, colorgrad::blues, colorgrad::greens, colorgrad::greys, colorgrad::oranges, colorgrad::purples, colorgrad::reds, colorgrad::br_bg, colorgrad::pr_gn, colorgrad::pi_yg, colorgrad::pu_or, colorgrad::rd_bu, colorgrad::rd_gy, colorgrad::rd_yl_bu, colorgrad::rd_yl_gn, colorgrad::bu_gn, colorgrad::bu_pu, colorgrad::gn_bu, colorgrad::or_rd, colorgrad::pu_bu_gn, colorgrad::pu_bu, colorgrad::pu_rd, colorgrad::rd_pu, colorgrad::rd_yl_gn, colorgrad::yl_gn_bu, colorgrad::yl_gn, colorgrad::yl_or_br, colorgrad::yl_or_rd];
        let gradient_map: HashMap<i32, fn() -> Gradient> = {
            let mut map: HashMap<i32, fn() -> Gradient> = HashMap::new();
            for i in 0..KEYS.len() {
                map.insert(KEYS[i], presets[i]);
            }
            map
        };

        /*fn get_func_name<F>(_: F) -> &'static str where F: Fn() -> Gradient {
            std::any::type_name::<F>()
        }
        let texts = {
            let mut texts = Vec::new();
            for i in 0..KEYS.len() {
                texts.push(get_func_name(presets[i]));
            }
            texts
        };*/
        let texts = ["cubehelix", "inferno", "magma", "turbo", "cividis", "sinebow", "rainbow", "warm", "cool", "plasma", "viridis", "spectral", "blues", "greens", "greys", "oranges", "purples", "reds", "br_bg", "pr_gn", "pi_yg", "pu_or", "rd_bu", "rd_gy", "rd_yl_bu", "rd_yl_gn", "bu_gn", "bu_pu", "gn_bu", "or_rd", "pu_bu_gn", "pu_bu", "pu_rd", "rd_pu", "rd_yl_gn", "yl_gn_bu", "yl_gn", "yl_or_br", "yl_or_rd"];
        let text_map = {
            let mut map = HashMap::new();
            for i in 0..KEYS.len() {
                map.insert(KEYS[i], texts[i].to_string());
            }
            map
        };

        Some(Self {
            show_cpu: false,
            show_gpu: true,
            dirty: true,
            mandelbrot_texture_id,
            julia_texture_id,
            mandelbrot_points: Arc::new(mandelbrot_vertices()),
            julia_points: Arc::new(julia_vertices()),
            last_selected: 0,
            selected: 0,
            text_map,
            gradient_map,
            max_iterations: MAX_ITERATIONS,
            show_mandelbrot: true,
            show_julia: false,
            c: [0.0, 0.0],
        })
    }
}

fn mandelbrot_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(6);
    vertices.push(Vertex {
        position: [-2.0, -1.25],
    });
    vertices.push(Vertex {
        position: [0.5, -1.25],
    });
    vertices.push(Vertex {
        position: [0.5, 1.25],
    });
    vertices.push(Vertex {
        position: [-2.0, -1.25],
    });
    vertices.push(Vertex {
        position: [0.5, 1.25],
    });
    vertices.push(Vertex {
        position: [-2.0, 1.25],
    });
    vertices
}

fn julia_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(6);
    vertices.push(Vertex {
        position: [-2.0, -2.0]
    });
    vertices.push(Vertex {
        position: [2.0, -2.0]
    });
    vertices.push(Vertex {
        position: [2.0, 2.0]
    });
    vertices.push(Vertex {
        position: [-2.0, -2.0]
    });
    vertices.push(Vertex {
        position: [2.0, 2.0]
    });
    vertices.push(Vertex {
        position: [-2.0, 2.0]
    });
    vertices
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.toggle_value(&mut self.show_mandelbrot, "Mandelbrot");
                ui.toggle_value(&mut self.show_julia, "Julia");
                ui.label("max_iterations");
                ui.add(egui::Slider::new(&mut self.max_iterations, 128..=MAX_ITERATIONS).step_by(128.0));
                // ui.toggle_value(&mut self.show_cpu, "CPU");
                // ui.toggle_value(&mut self.show_gpu, "GPU");
                ui.label("color gradient");
                egui::ComboBox::from_label("")
                    .selected_text(self.text_map.get(&self.selected).unwrap_or(&"None".to_string()))
                    // .selected_text(format!("{:?}", self.selected))
                    .show_ui(ui, |ui| {
                        for key in KEYS {
                            ui.selectable_value(&mut self.selected, key, self.text_map.get(&key).unwrap_or(&"None".to_string()));
                        }
                    });
                if self.show_julia {
                    ui.label("c.Re");
                    ui.add(egui::Slider::new(&mut self.c[0], -2.0..=2.0).step_by(0.01));
                    ui.label("c.Im");
                    ui.add(egui::Slider::new(&mut self.c[1], -2.0..=2.0).step_by(0.01));
                }
            });

            if self.show_mandelbrot {
                let mut bounds = PlotBounds::NOTHING;
                let resp = egui::plot::Plot::new("Mandelbrot_plot")
                    .legend(Legend::default())
                    // Must set margins to zero or the image and plot bounds will
                    // constantly fight, expanding the plot to infinity.
                    .set_margin_fraction(Vec2::new(0.0, 0.0))
                    .include_x(-2.0)
                    .include_x(0.5)
                    .include_y(-1.25)
                    .include_y(1.25)
                    .show(ui, |ui| {
                        bounds = ui.plot_bounds();

                        if self.show_gpu {
                            // Render the plot texture filling the viewport.
                            ui.image(
                                PlotImage::new(
                                    self.mandelbrot_texture_id,
                                    bounds.center(),
                                    [bounds.width() as f32, bounds.height() as f32],
                                )
                                    .name("Mandelbrot set (GPU)"),
                            );
                        }
                    });
                // Update the texture handle in egui from the previously
                // rendered texture (from the last frame).
                let wgpu_render_state = frame.wgpu_render_state().unwrap();
                let mut renderer = wgpu_render_state.renderer.write();

                let util: &mut MandelbrotRenderUtils = renderer.paint_callback_resources.get_mut().unwrap();

                if self.max_iterations != util.max_iterations() {
                    self.dirty = true;
                    util.set_max_iterations(self.max_iterations);
                }

                if self.selected != self.last_selected {
                    self.dirty = true;
                    let preset: &fn() -> Gradient = self.gradient_map.get(&self.selected).unwrap_or(&(colorgrad::cubehelix_default as fn() -> Gradient));
                    let grad = preset().sharp(COLOR_NUM, 0.);
                    let rgba_array: [[f32; 4]; COLOR_NUM] = grad.colors(COLOR_NUM).iter().map(|c| [c.r as f32, c.g as f32, c.b as f32, c.a as f32]).collect::<Vec<[f32; 4]>>().try_into().unwrap();
                    util.set_palette(rgba_array);
                }


                // Add a callback to egui to render the plot contents to
                // texture.
                ui.painter().add(mandelbrot::egui_wgpu_callback(
                    bounds,
                    Arc::clone(&self.mandelbrot_points),
                    resp.response.rect,
                    self.dirty,
                ));


                let texture_view = util.create_view();

                renderer.update_egui_texture_from_wgpu_texture(
                    &wgpu_render_state.device,
                    &texture_view,
                    wgpu::FilterMode::Linear,
                    self.mandelbrot_texture_id,
                );
            }

            if self.show_julia {
                let mut bounds = PlotBounds::NOTHING;
                let resp = egui::plot::Plot::new("Julia_plot")
                    .legend(Legend::default())
                    // Must set margins to zero or the image and plot bounds will
                    // constantly fight, expanding the plot to infinity.
                    .set_margin_fraction(Vec2::new(0.0, 0.0))
                    .include_x(-2.0)
                    .include_x(2.0)
                    .include_y(-2.0)
                    .include_y(2.0)
                    .show(ui, |ui| {
                        bounds = ui.plot_bounds();

                        if self.show_gpu {
                            // Render the plot texture filling the viewport.
                            ui.image(
                                PlotImage::new(
                                    self.julia_texture_id,
                                    bounds.center(),
                                    [bounds.width() as f32, bounds.height() as f32],
                                )
                                    .name("Julia set (GPU)"),
                            );
                        }
                    });
                // Update the texture handle in egui from the previously
                // rendered texture (from the last frame).
                let wgpu_render_state = frame.wgpu_render_state().unwrap();
                let mut renderer = wgpu_render_state.renderer.write();

                let util: &mut JuliaRenderUtils = renderer.paint_callback_resources.get_mut().unwrap();

                if self.max_iterations != util.max_iterations() {
                    self.dirty = true;
                    util.set_max_iterations(self.max_iterations);
                }

                if self.selected != self.last_selected {
                    self.dirty = true;
                    let preset: &fn() -> Gradient = self.gradient_map.get(&self.selected).unwrap_or(&(colorgrad::cubehelix_default as fn() -> Gradient));
                    let grad = preset().sharp(COLOR_NUM, 0.);
                    let rgba_array: [[f32; 4]; COLOR_NUM] = grad.colors(COLOR_NUM).iter().map(|c| [c.r as f32, c.g as f32, c.b as f32, c.a as f32]).collect::<Vec<[f32; 4]>>().try_into().unwrap();
                    util.set_palette(rgba_array);
                }

                if self.c != util.c() {
                    self.dirty = true;
                    util.set_c(self.c);
                }

                // Add a callback to egui to render the plot contents to
                // texture.
                ui.painter().add(julia::egui_wgpu_callback(
                    bounds,
                    Arc::clone(&self.julia_points),
                    resp.response.rect,
                    self.dirty,
                ));


                let texture_view = util.create_view();

                renderer.update_egui_texture_from_wgpu_texture(
                    &wgpu_render_state.device,
                    &texture_view,
                    wgpu::FilterMode::Linear,
                    self.julia_texture_id,
                );
            }

            self.dirty = false;
            self.last_selected = self.selected;
        });
    }
}

fn main() {
    let grad = colorgrad::cubehelix_default().sharp(COLOR_NUM, 0.);
    // let colors=grad.take(1000).collect::<Vec<_>>();
    let colors = grad.colors(COLOR_NUM);
    let mut rgba_array = [[0.0; 4]; COLOR_NUM];
    for (i, c) in colors.iter().enumerate() {
        rgba_array[i] = [c.r as f32, c.g as f32, c.b as f32, c.a as f32];
    }
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(1024.0, 1024.0)),
        centered: true,
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: WgpuConfiguration {
            // supported_backends: wgpu::Backends::DX12,
            device_descriptor: Arc::new(|adapter| {
                let base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                };

                wgpu::DeviceDescriptor {
                    label: Some("egui wgpu device"),
                    features: wgpu::Features::default() | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                    limits: wgpu::Limits {
                        // When using a depth buffer, we have to be able to create a texture
                        // large enough for the entire surface, and we want to support 4k+ displays.
                        max_texture_dimension_2d: 32768,
                        ..base_limits
                    },
                }
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    let app_creator: Box<dyn FnOnce(&CreationContext<'_>) -> Box<dyn App>> = Box::new(move |cc| Box::new(MyApp::new(cc, rgba_array).unwrap()));
    eframe::run_native(
        "Fractal Plotter",
        native_options,
        app_creator,
    ).expect("TODO: panic message");
}
