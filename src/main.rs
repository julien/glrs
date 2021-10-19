use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new();

    let context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let context = unsafe { context.make_current().unwrap() };

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
