extern crate gl;
use super::utils;
use gl::types::*;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use glutin::GlProfile;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;

static VS_SRC: &str = "
#version 150
in vec3 position;
in vec3 color;
out vec3 v_color;

void main() {
    gl_Position = vec4(position,  1.0);
    v_color = color;
}";

static FS_SRC: &str = "
#version 150
in vec3 v_color;
out vec4 out_color;

void main() {
    out_color = vec4(v_color, 1.0);
}";

pub fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title(" ");

    let context = ContextBuilder::new()
        .with_gl_profile(GlProfile::Core)
        .build_windowed(wb, &el)
        .unwrap();
    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|symbol| context.get_proc_address(symbol));

    utils::center_window(&context);

    let vertices: Vec<f32> = vec![
        0.5, -0.5, 0.0, 1.0, 0.0, 0.0, -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0,
    ];

    let vs = utils::compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = utils::compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = utils::link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;

    #[allow(temporary_cstring_as_ptr)]
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::UseProgram(program);

        let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr as GLuint,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<GLfloat>()) as GLint,
            ptr::null(),
        );

        let col_attr = gl::GetAttribLocation(program, CString::new("color").unwrap().as_ptr());
        gl::EnableVertexAttribArray(col_attr as GLuint);
        gl::VertexAttribPointer(
            col_attr as GLuint,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * mem::size_of::<GLfloat>()) as GLint,
            (3 * mem::size_of::<GLfloat>()) as *const GLvoid,
        );
    }

    el.run(move |event, _, control_flow| {
        context.window().request_redraw();

        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    unsafe {
                        gl::DeleteProgram(program);
                        gl::DeleteShader(vs);
                        gl::DeleteShader(fs);
                        gl::DeleteBuffers(1, &vbo);
                        gl::DeleteVertexArrays(1, &vao);
                    }
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::Resized(physical_size) => context.resize(physical_size),
                _ => (),
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::DrawArrays(gl::TRIANGLES, 0, 3);
                }
                context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}
