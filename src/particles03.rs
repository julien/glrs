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

static TARGET_FPS: u64 = 60;

static VS_SRC: &str = "
#version 330

layout(location=0) in vec2 a_position;
layout(location=1) in float a_pointsize;

void main() {
    gl_Position = vec4(vec3(a_position, 1.0), 1.0);
    gl_PointSize = a_pointsize;
}
";

static FS_SRC: &str = "
#version 330

out vec4 out_color;

void main() {
  out_color = vec4(1.0);
}
";

struct Mouse {
    x: f32,
    y: f32,
    r: f32,
    nx: f32,
    ny: f32,
    nr: f32,
    ex: f32,
    ey: f32,
    er: f32,
    client_x: f32,
    client_y: f32,
}

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

    let max_particles = 7000;
    let mut vertices: Vec<f32> = Vec::new();
    let particle_size = 7;
    let damp = -0.96;
    let mut rng = rand::thread_rng();

    let max_mice = 8;
    let mut mice: Vec<Mouse> = Vec::new();
    for _ in 0..max_mice {
        mice.push(Mouse {
            x: -1.0 + rng.gen::<f32>() * (1.0 - -1.0),
            y: -1.0 + rng.gen::<f32>() * (1.0 - -1.0),
            r: 0.1,
            nx: -1.0 + rng.gen::<f32>() * (1.0 - -1.0),
            ny: -1.0 + rng.gen::<f32>() * (1.0 - -1.0),
            nr: 0.1 + rng.gen::<f32>() * (1.0 - 0.1),
            ex: 0.02 + rng.gen::<f32>() * (0.01 - 0.02),
            ey: 0.02 + rng.gen::<f32>() * (0.01 - 0.02),
            er: 0.02 + rng.gen::<f32>() * (0.01 - 0.02),
            client_x: 0.0,
            client_y: 0.0,
        });
    }

    for _ in 0..max_particles {
        // position (x, y)
        vertices.push(-1.0 + rng.gen::<f32>() * (1.0 - -1.0));
        vertices.push(-1.0 + rng.gen::<f32>() * (1.0 - -1.0));
        // pointsize (r)
        vertices.push(1.0 + (rng.gen::<f32>() * (4.0 - 1.0)).floor() as f32);
        // velocity
        vertices.push(-0.03 + rng.gen::<f32>() * (0.03 - -0.03));
        vertices.push(-0.03 + rng.gen::<f32>() * (0.03 - -0.03));
        // acceleration
        vertices.push(-0.0009 + rng.gen::<f32>() * (-0.0002 - -0.0009));
        vertices.push(-0.0009 + rng.gen::<f32>() * (-0.0002 - -0.0009));
    }

    let mut vao = 0;
    let mut vertex_vbo = 0;
    let u_resolution;

    #[allow(temporary_cstring_as_ptr)]
    unsafe {
        gl::UseProgram(program);
        gl::Enable(gl::PROGRAM_POINT_SIZE);

        u_resolution =
            gl::GetUniformLocation(program, CString::new("u_resolution").unwrap().as_ptr());

        gl::Uniform2f(u_resolution, width as f32, height as f32);

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        gl::GenBuffers(1, &mut vertex_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const GLvoid,
            gl::DYNAMIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            (particle_size * std::mem::size_of::<f32>()) as gl::types::GLint,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            1,
            gl::FLOAT,
            gl::FALSE,
            (particle_size * std::mem::size_of::<f32>()) as gl::types::GLint,
            (2 * std::mem::size_of::<f32>()) as *const GLvoid,
        );
        gl::EnableVertexAttribArray(1);
    }

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
            Event::MainEventsCleared => {
                let num_particles = vertices.len() / particle_size;

                let mut i = 0;
                while i < vertices.len() {
                    vertices[i + 3] += vertices[i + 5];
                    vertices[i + 4] += vertices[i + 6];

                    vertices[i + 5] *= 0.0;
                    vertices[i + 6] *= 0.0;

                    for mouse in &mice {
                        let dx = vertices[i] - mouse.x;
                        let dy = vertices[i + 1] - mouse.y;
                        let dist = (dx * dx + dy * dy).sqrt();

                        if dist < mouse.r {
                            vertices[i] = mouse.x + dx / dist * mouse.r;
                            vertices[i + 1] = mouse.y + dy / dist * mouse.r;
                        } else {
                            while vertices[i + 2] > 2.0 {
                                vertices[i + 2] -= 0.1;
                            }
                        }
                    }

                    vertices[i] += vertices[i + 3];
                    vertices[i + 1] += vertices[i + 4];

                    if vertices[i] > 1.0 {
                        vertices[i] = 1.0;
                        vertices[i + 3] *= damp;
                    } else if vertices[i] < -1.0 {
                        vertices[i] = -1.0;
                        vertices[i + 3] *= damp;
                    }

                    if vertices[i + 1] > 1.0 {
                        vertices[i + 1] = 1.0;
                        vertices[i + 4] *= damp;
                    } else if vertices[i + 1] < -1.0 {
                        vertices[i + 1] = -1.0;
                        vertices[i + 4] *= damp;
                    }
                    i += 7;
                }

                for mouse in mice.iter_mut() {
                    if !update_mouse(mouse) {
                        mouse.ex = 0.07 + rng.gen::<f32>() * (0.01 - 0.07);
                        mouse.ey = 0.05 + rng.gen::<f32>() * (0.01 - 0.05);
                        mouse.er = 0.01 + rng.gen::<f32>() * (0.05 - 0.01);

                        mouse.nx = -0.9 + rng.gen::<f32>() * (1.0 - -0.9);
                        mouse.ny = -0.9 + rng.gen::<f32>() * (1.0 - -0.9);
                        mouse.nr = 0.01 + rng.gen::<f32>() * (0.3 - 0.01);
                    }
                }

                unsafe {
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        vertices.as_ptr() as *const GLvoid,
                    );

                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::DrawArrays(gl::POINTS, 0, num_particles as i32);
                }
            }
            _ => (),
        }
    });
}

fn update_mouse(mouse: &mut Mouse) -> bool {
    let mut vx = mouse.nx - mouse.x;
    let mut vy = mouse.ny - mouse.y;
    let mut vr = mouse.nr - mouse.r;

    let ax = vx.abs() > 0.1;
    let ay = vy.abs() > 0.1;
    let ar = vr.abs() > 0.1;

    if ax && ay && ar {
        vx *= mouse.ex;
        vy *= mouse.ey;
        vr *= mouse.er;

        mouse.x += vx;
        mouse.y += vy;
        mouse.r += vr;
        true
    } else {
        false
    }
}
