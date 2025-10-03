use std::ffi::CStr;
use std::num::NonZeroU32;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::ContextAttributesBuilder;
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("OpenGL in Rust on WSL");

    let window = window_builder.build(&event_loop).unwrap();
    let raw_window = unsafe { window.raw_window_handle() };
    let raw_display = unsafe { window.raw_display_handle() };

    // Create the GL display with preference (EGL first, fallback to GLX)
    let gl_display = unsafe {
        Display::new(
            raw_display,
            DisplayApiPreference::EglThenGlx(Box::new(|_| {})),
        )
        .unwrap()
    };

    // Build and use the config template
    let config_template = ConfigTemplateBuilder::new().with_alpha_size(8).build();
    let config = unsafe { gl_display.find_configs(config_template).unwrap().next().unwrap() };

    // Build context attributes
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window));

    // Create the GL context
    let not_current_context = unsafe { gl_display.create_context(&config, &context_attributes).unwrap() };

    // Build surface attributes for the window
    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window,
        NonZeroU32::new(1024).unwrap(),
        NonZeroU32::new(768).unwrap(),
    );

    // Create the window surface
    let gl_surface = unsafe { gl_display.create_window_surface(&config, &surface_attributes).unwrap() };

    // Make the context current on the surface
    let current_context = unsafe { not_current_context.make_current(&gl_surface).unwrap() };

    // Load OpenGL function pointers
    gl::load_with(|symbol| {
        let c_string = std::ffi::CString::new(symbol).unwrap();
        gl_display.get_proc_address(&c_string) as *const _
    });

    // Print OpenGL version
    unsafe {
        let version = CStr::from_ptr(gl::GetString(gl::VERSION) as *const i8).to_str().unwrap();
        println!("OpenGL version: {}", version);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                if size.width != 0 && size.height != 0 {
                    gl_surface.resize(
                        &current_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(0.2, 0.3, 0.3, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }
                gl_surface.swap_buffers(&current_context).unwrap();
            },
            _ => (),
        }
    });
}