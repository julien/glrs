extern crate gl;

use gl::types::*;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;

use rand::Rng;

use super::utils;

static VS_SRC: &'static str = "
#version 330

layout(location=0) in vec3 squareVertices;
layout(location=1) in vec4 xyzs;

uniform vec2 u_resolution;
uniform float u_time;

void main() {
	vec2 vp = xyzs.xy + squareVertices.xy;
	vec2 zeroToOne = vp / u_resolution;
	vec2 zeroToTwo = zeroToOne * 2.0;
	vec2 clipSpace = zeroToTwo - 1.0;

	gl_Position = vec4(clipSpace * vec2(1.0, -1.0), 0.0, 1.0);
}
";

static FS_SRC: &'static str = "
#version 330

out vec4 frag_color;

void main() {
  frag_color = vec4(0.6, 0.6, 0.6, 1.0);
}
";

#[derive(Debug, Copy, Clone)]
struct Particle {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    size: f32,
    life: f32,
}

pub fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title(" ");

    let context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|symbol| context.get_proc_address(symbol));

    let max_particles = 100;

    let vs = utils::compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = utils::compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = utils::link_program(vs, fs);

    let vertex_buffer_data: [GLfloat; 12] = [
        -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0,
    ];

    let size = context.window().inner_size();
    let mut rng = rand::thread_rng();
    let mut vao = 0;
    let mut vertex_vbo = 0;
    let mut position_vbo = 0;
    let mut u_resolution;
    let mut u_time;

    #[allow(temporary_cstring_as_ptr)]
    unsafe {
        gl::GenBuffers(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vertex_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertex_buffer_data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&vertex_buffer_data[0]),
            gl::STATIC_DRAW,
        );

        gl::GenBuffers(1, &mut position_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, position_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (max_particles * 4 * mem::size_of::<GLfloat>()) as GLsizeiptr,
            ptr::null(),
            gl::STREAM_DRAW,
        );

        gl::UseProgram(program);

        u_resolution =
            gl::GetUniformLocation(program, CString::new("u_resolution").unwrap().as_ptr());
        u_time = gl::GetUniformLocation(program, CString::new("u_time").unwrap().as_ptr());

        gl::Uniform2f(u_resolution, size.width as f32, size.height as f32);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let mut particles = vec![
        Particle {
            x: rng.gen_range(0.0..size.width as f32),
            y: rng.gen_range(0.0..size.height as f32),
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            size: -1.0,
            life: -1.0,
        };
        max_particles
    ];

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    unsafe {
                        gl::DeleteProgram(program);
                        gl::DeleteShader(vs);
                        gl::DeleteShader(fs);
                        gl::DeleteBuffers(1, &vertex_vbo);
                        gl::DeleteBuffers(1, &position_vbo);
                        gl::DeleteVertexArrays(1, &vao);
                    }
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    // gl::ClearColor(0.3, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }
                context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
