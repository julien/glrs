extern crate gl;
use super::utils;
use gl::types::*;
use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlProfile};
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::time::Instant;

static VS_SRC: &str = "
#version 330
layout(location=0) in vec2 a_position;
out vec2 v_texcoord;

void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
    v_texcoord = -0.5 + a_position + 1.0;
}
";

static FS_SRC: &str = "
#version 330
uniform sampler2D u_sampler;
uniform vec2 u_resolution;
uniform float u_time;

in vec2 v_texcoord;
out vec4 out_color;

void main() {
    vec2 uv = gl_FragCoord.xy / u_resolution;
    vec2 direction = vec2(uv.xy - vec2(0.5, 0.5));
    uv *= length(direction);
    uv.x *= cos(u_time * 0.08);
    uv.y += sin(u_time * 0.03);

    out_color = texture(u_sampler, uv);
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

    utils::center_window(&context);

    let vs = utils::compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = utils::compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = utils::link_program(vs, fs);

    let vertices: Vec<f32> = vec![
        0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5,
    ];

    let mut vao = 0;
    let mut vertex_vbo = 0;
    let mut texture = 0;
    let u_resolution;
    let u_time;

    #[allow(temporary_cstring_as_ptr)]
    unsafe {
        gl::UseProgram(program);

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vertex_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        let a_position =
            gl::GetAttribLocation(program, CString::new("a_position").unwrap().as_ptr());
        gl::EnableVertexAttribArray(a_position as GLuint);
        gl::VertexAttribPointer(a_position as u32, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());

        let u_sampler =
            gl::GetUniformLocation(program, CString::new("u_sampler").unwrap().as_ptr());
        utils::load_texture("bricks.png", &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::Uniform1i(u_sampler, 0);

        u_resolution =
            gl::GetUniformLocation(program, CString::new("u_resolution").unwrap().as_ptr());

        u_time = gl::GetUniformLocation(program, CString::new("u_time").unwrap().as_ptr());

        gl::Uniform2f(u_resolution, width as f32, height as f32);
    }

    let start_time = Instant::now();

    el.run(move |event, _, control_flow| {
        context.window().request_redraw();

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
                }
            }
            Event::RedrawRequested(_) => {
                context.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => unsafe {
                let elapsed_duration = Instant::now().duration_since(start_time).as_secs_f32();

                gl::Uniform1f(u_time, elapsed_duration);

                gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);
            },
            _ => (),
        }
    });
}
