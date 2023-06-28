mod util;

use std::sync::Arc;
use eframe::{App, CreationContext, egui::{self, Context, plot::{Legend, PlotBounds, PlotImage}}, egui_wgpu::WgpuConfiguration, emath::Vec2, epaint::{self}, Frame, wgpu};
use crate::util::{RenderUtils, Vertex};

const COLOR_NUM: usize = 128;

pub struct MyApp {
    show_cpu: bool,
    show_gpu: bool,
    dirty: bool,
    texture_id: epaint::TextureId,
    points: Arc<Vec<Vertex>>,
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

        Some(Self {
            show_cpu: false,
            show_gpu: true,
            dirty: true,
            texture_id,
            points: Arc::new(default_vertices()),
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
                // for (l, v, range) in [
                //     ("σ", &mut new_sigma, 0.0..=20.0),
                //     ("ρ", &mut new_rho, 0.0..=50.0),
                //     ("β", &mut new_beta, 0.0..=10.0),
                // ] {
                //     ui.label(l);
                //     ui.add(egui::Slider::new(v, range).step_by(0.01));
                // }

                ui.toggle_value(&mut self.show_cpu, "CPU");
                ui.toggle_value(&mut self.show_gpu, "GPU");
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
                // Add a callback to egui to render the plot contents to
                // texture.
                ui.painter().add(util::egui_wgpu_callback(
                    bounds,
                    Arc::clone(&self.points),
                    resp.response.rect,
                    self.dirty,
                ));

                // Update the texture handle in egui from the previously
                // rendered texture (from the last frame).
                let wgpu_render_state = frame.wgpu_render_state().unwrap();
                let mut renderer = wgpu_render_state.renderer.write();

                let plot: &RenderUtils = renderer.paint_callback_resources.get().unwrap();
                let texture_view = plot.create_view();

                renderer.update_egui_texture_from_wgpu_texture(
                    &wgpu_render_state.device,
                    &texture_view,
                    wgpu::FilterMode::Linear,
                    self.texture_id,
                );

                self.dirty = false;
            }
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
