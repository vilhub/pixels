use egui::{ClippedMesh, Context, TextureId, TexturesDelta, Vec2};
use egui_wgpu_backend::{wgpu::TextureViewDescriptor, BackendError, RenderPass, ScreenDescriptor};
use pixels::{wgpu, PixelsContext};
use winit::window::Window;

use crate::{HEIGHT, WIDTH};

/// Manages all state required for rendering egui over `Pixels`.
pub(crate) struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedMesh>,
    textures: TexturesDelta,

    // State for the GUI
    gui: Gui,
}

/// Example application state. A real application will need a lot more state than this.
struct Gui {
    /// Only show the egui window when true.
    window_open: bool,
    texture_id: TextureId,
    texture_size: Vec2,
    slider_value: f32,
}

impl Framework {
    /// Create egui.
    pub(crate) fn new(width: u32, height: u32, scale_factor: f32, pixels: &pixels::Pixels) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let egui_state = egui_winit::State::from_pixels_per_point(max_texture_size, scale_factor);
        let screen_descriptor = ScreenDescriptor {
            physical_width: width,
            physical_height: height,
            scale_factor,
        };
        let mut rpass = RenderPass::new(pixels.device(), pixels.render_texture_format(), 1);
        let textures = TexturesDelta::default();
        let texture = pixels.texture();
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let egui_texture = RenderPass::egui_texture_from_wgpu_texture(
            &mut rpass,
            pixels.device(),
            &texture_view,
            wgpu::FilterMode::Nearest,
        );
        let gui = Gui::new(
            egui_texture,
            Vec2 {
                x: WIDTH as f32,
                y: HEIGHT as f32,
            },
        );

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub(crate) fn handle_event(&mut self, event: &winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    /// Resize egui.
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.physical_width = width;
            self.screen_descriptor.physical_height = height;
        }
    }

    /// Update scaling factor.
    pub(crate) fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.scale_factor = scale_factor as f32;
    }

    /// Prepare egui.
    pub(crate) fn prepare(&mut self, window: &Window) -> f32 {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let mut menubar_width: f32 = 0.;
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            menubar_width = self.gui.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
        menubar_width
    }

    /// Render egui.
    pub(crate) fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) -> Result<(), BackendError> {
        // Upload all resources to the GPU.
        self.rpass
            .add_textures(&context.device, &context.queue, &self.textures)?;
        self.rpass.update_buffers(
            &context.device,
            &context.queue,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Record all render passes.
        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        )?;

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        self.rpass.remove_textures(textures)
    }
}

impl Gui {
    /// Create a `Gui`.
    fn new(texture_id: TextureId, texture_size: Vec2) -> Self {
        Self {
            window_open: true,
            texture_id: texture_id,
            texture_size: texture_size,
            slider_value: 0.
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context) -> f32 {
        let inner_response = egui::SidePanel::left("menubar_container").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.slider_value, 0.0..=100.0));
        });
        let menubar_width = inner_response.response.rect.width();

        let frame = egui::Frame {
            fill: ctx.style().visuals.window_fill(),
            ..egui::Frame::default()
        };
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.image(self.texture_id, self.texture_size);
        });

        egui::Window::new("Hello, egui!")
            .open(&mut self.window_open)
            .show(ctx, |ui| {
                ui.label("This example demonstrates using egui with pixels.");
                ui.label("Made with 💖 in San Francisco!");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x /= 2.0;
                    ui.label("Learn more about egui at");
                    ui.hyperlink("https://docs.rs/egui");
                });
            });
        menubar_width
    }
}
