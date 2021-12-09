extern crate gl;
use gl::types::*;
use glutin::dpi::LogicalPosition;
use glutin::window::Window;
use glutin::{ContextWrapper, PossiblyCurrent};
use std::ffi::CString;
use std::ptr;
use std::str;

pub fn center_window(context: &ContextWrapper<PossiblyCurrent, Window>) {
    let window = context.window();
    let monitor = window.current_monitor().unwrap();
    let scale = monitor.scale_factor() as f32;
    let size = window.outer_size();
    let width = size.width;
    let height = size.height;

    let x = (((monitor.size().width as f32) * scale) - width as f32) / 2.0;
    let y = (((monitor.size().height as f32) * scale) - height as f32) / 2.0;

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
