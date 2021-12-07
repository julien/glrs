extern crate gl;
use super::utils;
use gl::types::*;
use glutin::dpi::LogicalPosition;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlProfile};
use rand::Rng;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::time::Instant;

static TARGET_FPS: u64 = 60;

static VS_SRC: &str = "
#version 410

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
    gl_PointSize = 30.0;
}
";

static FS_SRC: &str = "
#version 410

out vec4 out_color;

void main() {
  out_color = vec4(1.0);
}
";

#[derive(Clone)]
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

    let context = ContextBuilder::new()
        .with_gl_profile(GlProfile::Core)
        .with_vsync(true)
        .build_windowed(wb, &el)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    gl::load_with(|symbol| context.get_proc_address(symbol));

    let monitor = context.window().current_monitor().unwrap();
    let size = context.window().inner_size();

    let x = (monitor.size().width - size.width) / 2;
    let y = (monitor.size().height - size.height) / 2;

    context
        .window()
        .set_outer_position(LogicalPosition::new(x, y));

    let max_particles = 5000;

    let vs = utils::compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = utils::compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = utils::link_program(vs, fs);

    let vertex_buffer_data: [GLfloat; 12] = [
        -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0,
    ];

    let mut rng = rand::thread_rng();
    let mut vao = 0;
    let mut vertex_vbo = 0;
    let mut position_vbo = 0;
    let u_resolution;
    let u_time;
    let viewport_width = size.width as f32;
    let viewport_height = size.height as f32;

    #[allow(temporary_cstring_as_ptr)]
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
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
        gl::Enable(gl::PROGRAM_POINT_SIZE);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let mut particles = vec![
        Particle {
            x: rng.gen_range(0.0..viewport_width),
            y: rng.gen_range(0.0..viewport_height),
            z: 0.0,
            vx: rng.gen_range(-2.0..2.0),
            vy: rng.gen_range(-2.0..2.0),
            size: 1.0,
            life: 1.0,
        };
        max_particles
    ];

    for p in &mut particles {
        p.x = rng.gen_range(0.0..viewport_width);
        p.y = rng.gen_range(0.0..viewport_height);
        p.vx = rng.gen_range(-2.0..2.0);
        p.vy = rng.gen_range(-2.0..2.0);
    }

    let mut particles_data: Vec<GLfloat> = vec![0.0; max_particles * 4];

    let mut last_used_particle = 0;

    el.run(move |event, _, control_flow| {
        let start_time = Instant::now();

        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => {
                if event == WindowEvent::CloseRequested {
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
            }
            Event::RedrawRequested(_) => {
                context.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => {
                context.window().request_redraw();

                let elapsed_duration = Instant::now().duration_since(start_time).as_secs_f32();

                let mut new_particles = elapsed_duration * 10000.0;
                if new_particles >= 0.016 * 10000.0 {
                    new_particles = 0.016 * 10000.0;
                }

                for _ in 0..(new_particles as u32) {
                    last_used_particle =
                        find_free_particle(&particles, last_used_particle, max_particles as u32);

                    if let Some(p) = particles.get_mut(last_used_particle as usize) {
                        p.vx = rng.gen_range(-0.8..0.8);
                        p.vy = rng.gen_range(-0.8..0.8);
                        p.life = rng.gen_range(0.0..200.0);
                        p.size = 1.0;
                    }
                }

                let mut particles_count = 0;

                unsafe {
                    for mut p in &mut particles {
                        if p.life > 0.0 {
                            p.x += p.vx;
                            p.y += p.vy;

                            if p.x > viewport_width {
                                p.x = 0.0;
                            } else if p.x < 0.0 {
                                p.x = viewport_width;
                            }
                            if p.y > viewport_height {
                                p.y = 0.0;
                            } else if p.y < 0.0 {
                                p.y = viewport_height;
                            }

                            particles_data[4 * particles_count] = p.x;
                            particles_data[4 * particles_count + 1] = p.y;
                            particles_data[4 * particles_count + 2] = p.z;
                            particles_data[4 * particles_count + 3] = p.size;

                            p.life -= 0.01;
                        } else {
                            p.x = rng.gen_range(0.0..viewport_width);
                            p.y = rng.gen_range(0.0..viewport_height);
                            p.vx = rng.gen_range(-2.0..2.0);
                            p.vy = rng.gen_range(-2.0..2.0);
                            p.life = rng.gen_range(0.0..100.0);
                        }

                        particles_count += 1;
                    }

                    gl::BindBuffer(gl::ARRAY_BUFFER, position_vbo);
                    gl::BufferSubData(
                        gl::ARRAY_BUFFER,
                        0,
                        (particles_count * 4 * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        mem::transmute(&particles_data[0]),
                    );

                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                    gl::Uniform2f(u_resolution, viewport_width, viewport_height);
                    gl::Uniform1f(u_time, elapsed_duration);

                    gl::EnableVertexAttribArray(0);
                    gl::BindBuffer(gl::ARRAY_BUFFER, vertex_vbo);
                    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

                    gl::EnableVertexAttribArray(1);
                    gl::BindBuffer(gl::ARRAY_BUFFER, position_vbo);
                    gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, 0, ptr::null());

                    gl::VertexAttribDivisor(0, 0);
                    gl::VertexAttribDivisor(1, 1);

                    gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, 4, particles_count as i32);

                    gl::DisableVertexAttribArray(0);
                    gl::DisableVertexAttribArray(1);
                }
            }
            _ => (),
        }
    });
}

fn find_free_particle(particles: &[Particle], last_used_particle: u32, max_particles: u32) -> u32 {
    for index in last_used_particle..max_particles {
        if let Some(particle) = particles.get(index as usize) {
            if particle.life < 0.0 {
                return index;
            }
        }
    }
    for index in 0..last_used_particle {
        if let Some(particle) = particles.get(index as usize) {
            if particle.life < 0.0 {
                return index;
            }
        }
    }
    0
}
