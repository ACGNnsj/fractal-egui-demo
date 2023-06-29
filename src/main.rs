mod util;

use std::collections::HashMap;
use std::sync::Arc;
use colorgrad::Gradient;
use eframe::{App, CreationContext, egui::{self, Context, plot::{Legend, PlotBounds, PlotImage}}, egui_wgpu::WgpuConfiguration, emath::Vec2, epaint::{self}, Frame, wgpu};
use crate::util::{RenderUtils, Vertex};

const COLOR_NUM: usize = 128;
const KEYS: [i32; 7] = [0, 1, 2, 3, 4, 5, 6];
// static mut SELECTED: i32 =1;

pub struct MyApp {
    show_cpu: bool,
    show_gpu: bool,
    dirty: bool,
    texture_id: epaint::TextureId,
    points: Arc<Vec<Vertex>>,
    last_selected: i32,
    selected: i32,
    text_map: HashMap<i32, String>,
    // gradient_map: HashMap<String, dyn Fn() -> Gradient>,
    gradient_map: HashMap<i32, fn() -> Gradient>,
}

impl MyApp {
    pub fn new<'a>(cc: &'a CreationContext<'a>, palette: [[f32; 4]; COLOR_NUM]) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;
        let target_format = wgpu_render_state.target_format;

        let util = RenderUtils::new(device, target_format, palette);
        let texture_id = {
            let mut renderer = wgpu_render_state.renderer.write();
            renderer.register_native_texture(device, &util.create_view(), wgpu::FilterMode::Linear)
        };

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(util);

        let texts = ["cubehelix", "sinebow", "rainbow", "turbo", "cividis", "warm", "cool"];
        let mut text_map = {
            let mut map = HashMap::new();
            for i in 0..KEYS.len() {
                map.insert(KEYS[i], texts[i].to_string());
            }
            map
        };
        let presets = [colorgrad::cubehelix_default, colorgrad::sinebow, colorgrad::rainbow, colorgrad::turbo, colorgrad::cividis, colorgrad::warm, colorgrad::cool];
        let mut gradient_map: HashMap<i32, fn() -> Gradient> = {
            let mut map: HashMap<i32, fn() -> Gradient> = HashMap::new();
            for i in 0..KEYS.len() {
                map.insert(KEYS[i], presets[i]);
            }
            map
        };
        Some(Self {
            show_cpu: false,
            show_gpu: true,
            dirty: true,
            texture_id,
            points: Arc::new(default_vertices()),
            last_selected: 0,
            selected: 0,
            text_map,
            gradient_map,
        })
    }
}

fn default_vertices() -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(4);
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

impl App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.toggle_value(&mut self.show_cpu, "CPU");
                ui.toggle_value(&mut self.show_gpu, "GPU");
                egui::ComboBox::from_label("Select one!")
                    .selected_text(self.text_map.get(&self.selected).unwrap_or(&"None".to_string()))
                    // .selected_text(format!("{:?}", self.selected))
                    .show_ui(ui, |ui| {
                        for key in KEYS {
                            ui.selectable_value(&mut self.selected, key, self.text_map.get(&key).unwrap_or(&"None".to_string()));
                        }
                        /*for (selected, text) in self.text_map.iter() {
                            ui.selectable_value(&mut self.selected, *selected, text);
                        }*/
                    }, );
            });

            // if self.q != [new_sigma, new_rho, new_beta] {
            //     self.q = [new_sigma, new_rho, new_beta];
            // 
            //     self.points = Arc::new(default_vertices());
            //     self.dirty = true;
            // }

            let mut bounds = PlotBounds::NOTHING;
            let resp = egui::plot::Plot::new("my_plot")
                .legend(Legend::default())
                // Must set margins to zero or the image and plot bounds will
                // constantly fight, expanding the plot to infinity.
                .set_margin_fraction(Vec2::new(0.0, 0.0))
                // .include_x(-25.0)
                .include_x(-2.0)
                // .include_x(25.0)
                .include_x(0.5)
                // .include_y(0.0)
                .include_y(-1.25)
                // .include_y(60.0)
                .include_y(1.25)
                .show(ui, |ui| {
                    bounds = ui.plot_bounds();

                    if self.show_gpu {
                        // Render the plot texture filling the viewport.
                        ui.image(
                            PlotImage::new(
                                self.texture_id,
                                bounds.center(),
                                [bounds.width() as f32, bounds.height() as f32],
                            )
                                .name("Mandelbrot set (GPU)"),
                        );
                    }

                    if self.show_cpu {
                        ui.line(
                            egui::plot::Line::new(egui::plot::PlotPoints::from_iter(
                                self.points
                                    .iter()
                                    .map(|p| [p.position[0] as f64, p.position[1] as f64]),
                            ))
                                .name("Mandelbrot set (CPU)"),
                        );
                    }
                });

            if self.show_gpu {
                // Update the texture handle in egui from the previously
                // rendered texture (from the last frame).
                let wgpu_render_state = frame.wgpu_render_state().unwrap();
                let mut renderer = wgpu_render_state.renderer.write();

                let util: &mut RenderUtils = renderer.paint_callback_resources.get_mut().unwrap();

                if self.selected != self.last_selected {
                    self.dirty = true;
                    let preset: &fn() -> Gradient = self.gradient_map.get(&self.selected).unwrap_or(&(colorgrad::cubehelix_default as fn() -> Gradient));
                    let grad = preset().sharp(COLOR_NUM, 0.);
                    let rgba_array: [[f32; 4]; COLOR_NUM] = grad.colors(COLOR_NUM).iter().map(|c| [c.r as f32, c.g as f32, c.b as f32, c.a as f32]).collect::<Vec<[f32; 4]>>().try_into().unwrap();
                    util.set_palette(rgba_array);
                }


                // Add a callback to egui to render the plot contents to
                // texture.
                ui.painter().add(util::egui_wgpu_callback(
                    bounds,
                    Arc::clone(&self.points),
                    resp.response.rect,
                    self.dirty,
                ));


                let texture_view = util.create_view();

                renderer.update_egui_texture_from_wgpu_texture(
                    &wgpu_render_state.device,
                    &texture_view,
                    wgpu::FilterMode::Linear,
                    self.texture_id,
                );

                self.dirty = false;
            }
            self.last_selected = self.selected;
        });
    }
}

fn main() {
    let f = colorgrad::cubehelix_default;
    let grad = colorgrad::cubehelix_default().sharp(COLOR_NUM, 0.);
    // let colors=grad.take(1000).collect::<Vec<_>>();
    let colors = grad.colors(COLOR_NUM);
    let mut rgba_array = [[0.0; 4]; COLOR_NUM];
    for (i, c) in colors.iter().enumerate() {
        rgba_array[i] = [c.r as f32, c.g as f32, c.b as f32, c.a as f32];
    }
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(Vec2::new(900.0, 900.0)),
        centered: true,
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: WgpuConfiguration {
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
                        max_texture_dimension_2d: 8192,
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
