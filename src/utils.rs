extern crate gl;
use gl::types::*;
use glutin::dpi::LogicalPosition;
use glutin::window::Window;
use glutin::{ContextWrapper, PossiblyCurrent};
use stb_image::image::{load, LoadResult};
use std::ffi::c_void;
use std::ffi::CString;
use std::path::Path;
use std::ptr;
use std::str;

pub fn center_window(context: &ContextWrapper<PossiblyCurrent, Window>) {
    let window = context.window();
    let monitor = window.current_monitor().unwrap();
    let monitor_scale = monitor.scale_factor() as f32;
    let window_scale = window.scale_factor() as f32;
    let size = window.outer_size();
    let window_width = size.width as f32;
    let window_height = size.height as f32;
    let x = (((monitor.size().width as f32) / monitor_scale) - (window_width / window_scale)) / 2.0;
    let y =
        (((monitor.size().height as f32) / monitor_scale) - (window_height / window_scale)) / 2.0;

    window.set_outer_position(LogicalPosition::new(x, y));
}

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(ty);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf).expect("shader info log not valid utf8")
            );
        }
        shader
    }
}

pub fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf).expect("program info log not valid utf8")
            );
        }
        program
    }
}

pub fn lerp(norm: f32, min: f32, max: f32) -> f32 {
    (max - min) * norm + min
}

pub fn map(val: f32, srcmin: f32, srcmax: f32, dstmin: f32, dstmax: f32) -> f32 {
    lerp(norm(val, srcmin, srcmax), dstmin, dstmax)
}

pub fn norm(val: f32, min: f32, max: f32) -> f32 {
    (val - min) / (max - min)
}

pub fn load_texture(filename: &str, texture: &mut u32) {
    match load(Path::new(filename)) {
        LoadResult::Error(s) => {
            eprintln!("{}", s);
        }
        LoadResult::ImageU8(i) => {
            init_texture(
                texture,
                i.width as i32,
                i.height as i32,
                i.data.as_ptr() as *const c_void,
            );
        }
        LoadResult::ImageF32(i) => {
            init_texture(
                texture,
                i.width as i32,
                i.height as i32,
                i.data.as_ptr() as *const c_void,
            );
        }
    }
}

fn init_texture(texture: &mut u32, width: i32, height: i32, data: *const c_void) {
    unsafe {
        gl::GenTextures(1, texture);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, *texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width,
            height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    }
}
