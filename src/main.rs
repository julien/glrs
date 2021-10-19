extern crate gl;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new();

    let context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|symbol| context.get_proc_address(symbol));

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.3, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
