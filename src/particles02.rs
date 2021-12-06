extern crate gl;
use super::utils;
use gl::types::*;
use glutin::dpi::{LogicalPosition, PhysicalSize};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlProfile};
use rand::Rng;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::time::{Duration, Instant};

static TARGET_FPS: u64 = 60;

static VS_SRC: &str = "
#version 410

layout(location=0) in vec3 V;
uniform float T;
out float Z;

void main() {
    float N = 0.0;
    float O = 1.0;
    vec4 v = vec4(V, O);
    v.z += 2.0 * (sin(T * 2.0 + v.x) + cos(T * 2.0 + v.y * 1.5));
    gl_PointSize = 8.0 + sin(T * 3.5) + cos(T / 2.0);
    gl_Position = mat4(7, N, N, N,
                       N, 7, N, N,
                       N, N, -O, -O,
                       5.0 * sin(T), 5 * sin(T / 3.15), N, O) * v;
    Z = O - gl_Position.z / 8.0;
}
";

static FS_SRC: &str = "
#version 410

in float Z;
out vec4 out_color;

void main() {
  out_color = vec4(Z, Z, Z, 1.0);
}
";

pub fn main() {
    let width = 1024;
    let height = 768;

    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .with_resizable(false)
        .with_title(" ");

    let context = ContextBuilder::new()
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|symbol| context.get_proc_address(symbol));

    let monitor = context.window().current_monitor().unwrap();

    let x = (monitor.size().width - width) / 2;
    let y = (monitor.size().height - height) / 2;

    context
        .window()
        .set_outer_position(LogicalPosition::new(x, y));

    let vs = utils::compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = utils::compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = utils::link_program(vs, fs);

    let max_particles = 5000;
    let mut vertices: Vec<f32> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..max_particles {
        let x = rng.gen::<f32>() * 2.0 - 1.0;
        let y = rng.gen::<f32>() * 2.0 - 1.0;
        vertices.push(x);
        vertices.push(y);
        vertices.push(0.0);
        vertices.push(x);
        vertices.push(y);
        vertices.push(0.0);
    }

    let mut vao = 0;
    let mut vertex_vbo = 0;
    let u_time;
    let mut u_time_value: f32 = 0.0;

    #[allow(temporary_cstring_as_ptr)]
    unsafe {
        gl::UseProgram(program);
        gl::Enable(gl::PROGRAM_POINT_SIZE);

        u_time = gl::GetUniformLocation(program, CString::new("T").unwrap().as_ptr());

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vertex_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_vbo);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
        gl::EnableVertexAttribArray(0);
    }

    el.run(move |event, _, control_flow| {
        let mut start_time = Instant::now();

        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => {
                if event == WindowEvent::CloseRequested {
                    unsafe {
                        gl::DeleteProgram(program);
                        gl::DeleteShader(vs);
                        gl::DeleteShader(fs);
                        gl::DeleteBuffers(1, &vertex_vbo);
                        gl::DeleteVertexArrays(1, &vao);
                    }
                    *control_flow = ControlFlow::Exit
                } else {
                    start_time = Instant::now();
                    *control_flow = ControlFlow::Wait;
                }
            }
            Event::RedrawRequested(_) => {
                context.swap_buffers().unwrap();
            }
            _ => (),
        }

        match *control_flow {
            ControlFlow::Exit => (),
            _ => {
                context.window().request_redraw();

                let elapsed_duration = Instant::now().duration_since(start_time);

                let elapsed_time = elapsed_duration.as_millis() as u64;

                let wait_millis = match 1000 / TARGET_FPS >= elapsed_time {
                    true => 1000 / TARGET_FPS - elapsed_time,
                    false => 0,
                };

                let next = start_time + Duration::from_millis(wait_millis);

                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                for k in 0..max_particles {
                    let z = (((k as f32) + u_time_value * 2.0) % 30.0) - 15.0;
                    vertices[k * 6 + 2] = z - 0.003;
                    vertices[k * 6 + 5] = z + 0.003;
                }

                unsafe {
                    gl::BufferData(
                        gl::ARRAY_BUFFER,
                        (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        vertices.as_ptr() as *const GLvoid,
                        gl::STATIC_DRAW,
                    );
                    gl::Uniform1f(u_time, u_time_value);
                    gl::DrawArrays(gl::POINTS, 0, max_particles as i32);
                }

                u_time_value += 0.01;

                *control_flow = ControlFlow::WaitUntil(next);
            }
        }
    });
}
