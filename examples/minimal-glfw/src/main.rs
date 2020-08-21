#![deny(clippy::all)]
#![forbid(unsafe_code)]

use glfw::{Action, Key, WindowEvent};
use pixels::{Error, Pixels, SurfaceTexture};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("Failed to initialize GLFW.");
    let (mut window, events) = glfw
        .create_window(WIDTH, HEIGHT, "Hello Pixels", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    let mut hidpi_factor = window.get_content_scale();

    window.set_key_polling(true);

    let mut pixels = {
        let window_size = window.get_size();
        let width = (window_size.0 as f32 * hidpi_factor.0) as u32;
        let height = (window_size.1 as f32 * hidpi_factor.1) as u32;
        let surface_texture = SurfaceTexture::new(width, height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    // Close events
                    window.set_should_close(true);
                }
                WindowEvent::Size(width, height) => {
                    // Resize the window
                    let width = (width as f32 * hidpi_factor.0) as u32;
                    let height = (height as f32 * hidpi_factor.1) as u32;
                    pixels.resize(width, height);
                }
                WindowEvent::ContentScale(width, height) => {
                    // Update HiDPI scaling factor
                    hidpi_factor = (width, height);
                }
                _ => {}
            }
        }

        // Draw the current frame
        world.draw(pixels.get_frame());
        pixels.render()?;

        // Update internal state
        world.update();
    }

    Ok(())
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
