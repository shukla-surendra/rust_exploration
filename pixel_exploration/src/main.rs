use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn main() {
    // create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    // create pixel buffer
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // fill frame with solid blue
                let frame = pixels.frame_mut();
                for chunk in frame.chunks_exact_mut(4) {
                    chunk[0] = 0;     // R
                    chunk[1] = 0;     // G
                    chunk[2] = 255;   // B
                    chunk[3] = 255;   // A
                }

                pixels.render().unwrap();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
