use raqote::*;
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;



const BG_COLOR: u32 = 0xFF1E1F22;      // Main editor background
const PANEL_COLOR: u32 = 0xFF2B2D30;   // Side and bottom panel background
const BORDER_COLOR: u32 = 0xFF393B40;  // Subtle border color
const MARGIN: f32 = 10.0;              // Space between islands
const CORNER_RADIUS: f32 = 12.0;       // Rounded corner radius

fn draw_rounded_rect(dt: &mut DrawTarget, x: f32, y: f32, w: f32, h: f32, radius: f32, color: u32) {
    let mut pb = PathBuilder::new();
    pb.move_to(x + radius, y);
    pb.line_to(x + w - radius, y);
    pb.quad_to(x + w, y, x + w, y + radius);
    pb.line_to(x + w, y + h - radius);
    pb.quad_to(x + w, y + h, x + w - radius, y + h);
    pb.line_to(x + radius, y + h);
    pb.quad_to(x, y + h, x, y + h - radius);
    pb.line_to(x, y + radius);
    pb.quad_to(x, y, x + radius, y);
    let path = pb.finish();

    // Fill the island
    dt.fill(&path, &Source::Solid(SolidSource::from_unpremultiplied_argb(
        (color >> 24) as u8, (color >> 16) as u8, (color >> 8) as u8, color as u8
    )), &DrawOptions::new());

    // Draw the subtle border
    dt.stroke(&path, &Source::Solid(SolidSource::from_unpremultiplied_argb(0xFF, 0x39, 0x3B, 0x40)),
              &StrokeStyle::default(), &DrawOptions::new());
}

#[derive(Default)]
struct App<'a> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'a>>,
    // Store dimensions so you don't reallocate DrawTarget every frame
    width: u32,
    height: u32,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
            let size = window.inner_size();

            let surface = SurfaceTexture::new(size.width, size.height, Arc::clone(&window));
            // Initialize pixels with the CURRENT window size for 1:1 quality
            let pixels = Pixels::new(size.width, size.height, surface).unwrap();

            self.window = Some(window);
            self.pixels = Some(pixels);
            self.width = size.width;
            self.height = size.height;
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(new_size) => {
                if let Some(pixels) = &mut self.pixels {
                    // CRITICAL: Update both surface (GPU) and buffer (Resolution)
                    pixels.resize_surface(new_size.width, new_size.height).unwrap();
                    pixels.resize_buffer(new_size.width, new_size.height).unwrap();

                    // Update state so drawing logic knows the new bounds
                    self.width = new_size.width;
                    self.height = new_size.height;
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(pixels), Some(_)) = (&mut self.pixels, &self.window) {
                    // Create a DrawTarget that matches the current window resolution
                    let mut dt = DrawTarget::new(self.width as i32, self.height as i32);

                    // --- DYNAMIC UI LAYOUT ---
                    let w = self.width as f32;
                    let h = self.height as f32;
                    let margin = 10.0;

                    // Clear Background
                    dt.clear(SolidSource::from_unpremultiplied_argb(0xFF, 0x1E, 0x1F, 0x22));

                    // Dynamic Side Panel (File Management)
                    let side_w = (w * 0.2).max(150.0); // 20% of width or min 150px
                    draw_rounded_rect(&mut dt, margin, margin, side_w, h - (margin * 2.0), 12.0, 0xFF2B2D30);

                    // Dynamic Bottom Panel (Shell)
                    let bottom_h = (h * 0.25).max(100.0); // 25% of height
                    let bottom_x = side_w + (margin * 2.0);
                    let bottom_w = w - bottom_x - margin;
                    draw_rounded_rect(&mut dt, bottom_x, h - bottom_h - margin, bottom_w, bottom_h, 12.0, 0xFF2B2D30);

                    // Dynamic Code Area
                    let code_w = bottom_w;
                    let code_h = h - bottom_h - (margin * 3.0);
                    draw_rounded_rect(&mut dt, bottom_x, margin, code_w, code_h, 12.0, 0xFF1E1F22);

                    // --- RENDER ---
                    let frame = pixels.frame_mut();
                    for (dst, &src) in frame.chunks_exact_mut(4).zip(dt.get_data().iter()) {
                        let a = ((src >> 24) & 0xff) as u8;
                        let r = ((src >> 16) & 0xff) as u8;
                        let g = ((src >> 8) & 0xff) as u8;
                        let b = (src & 0xff) as u8;
                        dst.copy_from_slice(&[r, g, b, a]);
                    }
                    pixels.render().unwrap();
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            // This triggers the WindowEvent::RedrawRequested you wrote
            window.request_redraw();
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

