#[macro_use]
extern crate glium;
extern crate time;

mod math;

use glium::{DisplayBuild, Surface, Frame};
use glium::backend::glutin_backend::{WinRef, GlutinFacade};
use std::collections::HashMap;

struct Camera{
    position: [f32; 3],
    pitch: f32,
    yaw: f32,
}

struct Keyboard {
    keys: HashMap<glium::glutin::VirtualKeyCode, glium::glutin::ElementState>,
}

impl Keyboard {
    pub fn new() -> Keyboard{
        Keyboard{
            keys: HashMap::new(),
        }
    }

    pub fn is_key_down(&self, key: glium::glutin::VirtualKeyCode) -> glium::glutin::ElementState {
        match self.keys.get(&key) {
            Some(down) => *down,
            None => glium::glutin::ElementState::Released,
        }
    }

    pub fn set_key_state(&mut self, key: glium::glutin::VirtualKeyCode, state: glium::glutin::ElementState) {
        self.keys.insert(key, state);
    }
}

impl Camera {
    pub fn new(pitch: f32, yaw: f32, position: [f32; 3]) -> Camera{
        Camera{
            pitch: pitch,
            yaw: yaw,
            position: position,
        }
    }

    pub fn view_mat4(&self) -> [[f32; 4]; 4] {
        math::view_matrix_from_radians(self.pitch, self.yaw, self.position)
    }
}

#[derive(Copy, Clone)]
struct Vertex{
    position: [f32; 3],
}
implement_vertex!(Vertex, position);

impl Vertex {
    fn new(position: [f32; 3]) -> Vertex {
        Vertex{
            position: position,
        }
    }
}

struct Cube<T> {
    vertices: [Vertex; 8],
    indices: Vec<T>,
}

fn make_facade(window_dimensions: (u32, u32), depth_bit: u8, title: String, fullscreen: bool) -> GlutinFacade {
    let screen_size : (u32, u32) = glium::glutin::get_primary_monitor().get_dimensions();
    if fullscreen {
        let facade = match glium::glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(screen_size.0, screen_size.1)
            .with_decorations(false)
            .with_depth_buffer(depth_bit)
            .build_glium(){
                Ok(facade) => facade,
                Err(error) => panic!(error),
            };
        match facade.get_window(){
            Some(window) => window,
            None => panic!("Unable to find Window"),
        }.set_position(0,0);
        return facade;
    } else {
        let facade = match glium::glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(window_dimensions.0, window_dimensions.1)
            .with_depth_buffer(depth_bit)
            .build_glium(){
                Ok(facade) => facade,
                Err(error) => panic!(error),
            };
        match facade.get_window(){
            Some(window) => window,
            None => panic!("Unable to find Window"),
        }.set_position(((screen_size.0 as i32 - window_dimensions.0 as i32) / 2), ((screen_size.1 as i32 - window_dimensions.1 as i32) / 2));
        return facade;
    }
}

fn update_mouse_lock_position(resolution: (u32, u32), mouse_lock_ratio: (f32, f32)) -> (i32, i32) {
    ((resolution.0 as f32 * mouse_lock_ratio.0) as i32, (resolution.1 as f32 * mouse_lock_ratio.1) as i32)
}

fn run(debug: bool, x: f32, y: f32, z: f32, near: f32, far: f32, field_of_view: f32, d_pitch: f32, d_yaw: f32){
    let mut resolution : (u32, u32) = (640, 480);
    let mut mouse_delta : (f32, f32);
    let mut aspect_ratio = resolution.0 as f32 / resolution.1 as f32;

    let mouse_lock_ratio : (f32, f32) = (0.5, 0.5);

    let depth = 24;
    let title = "Smooth Voxel".to_string();

    let fullscreen = !debug;
    let facade: GlutinFacade = make_facade(resolution, depth, title, fullscreen);

    let mut locked = true;

    let window : WinRef = match facade.get_window(){
        Some(win) => win,
        None => panic!("Unable to find a Window"),
    };

    resolution = match window.get_inner_size(){
        Some(dimensions) => dimensions,
        None => panic!("Unable to find a Window"),
    };

    let mut mouse_lock_position : (i32, i32) = update_mouse_lock_position(resolution, mouse_lock_ratio);

    match window.set_cursor_state(glium::glutin::CursorState::Grab){
        Ok(()) => (),
        Err(error) => panic!(error),
    };

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;

        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 perspective;

        out vec3 o_pos;

        void main() {
            o_pos = position;
            gl_Position = vec4(position, 1.0) * model * view * perspective;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        in vec3 o_pos;

        void main() {
            color = vec4(o_pos, 1.0);
        }
    "#;

    let program = match glium::Program::from_source(&facade, vertex_shader_src, fragment_shader_src, None){
        Ok(program) => program,
        Err(error) => match error{
            glium::program::ProgramCreationError::CompilationError(string) => panic!("CompliationError: {}", string),
            glium::program::ProgramCreationError::LinkingError(string) => panic!("LinkingError: {}", string),
            glium::program::ProgramCreationError::ShaderTypeNotSupported => panic!("ShaderTypeNotSupported"),
            glium::program::ProgramCreationError::CompilationNotSupported => panic!("CompilationNotSupported"),
            glium::program::ProgramCreationError::TransformFeedbackNotSupported => panic!("TransformFeedbackNotSupported"),
            glium::program::ProgramCreationError::PointSizeNotSupported => panic!("PointSizeNotSupported"),
        }
    };

    let mut camera: Camera = Camera::new(0.0, 0.0, math::empty_vec3());

    let mut world_up = math::empty_vec3();
    world_up[1] = 1.0;

    let mut view_direction = math::empty_vec3();
    view_direction[2] = 1.0;

    let mut view = camera.view_mat4();

    let mut perspective = math::perspective_mat4(
        near,
        far,
        field_of_view,
        aspect_ratio,
    );

    let mut model = math::translation_mat4([
        x,
        y,
        z,
    ]);

    let mut uniforms = uniform!{
        model: model,
        view: view,
        perspective: perspective,
    };

    let cube : Cube<u32> = Cube{vertices: [
        Vertex::new([0.0, 0.0, 0.0]),
        Vertex::new([1.0, 0.0, 0.0]),
        Vertex::new([1.0, 1.0, 0.0]),
        Vertex::new([0.0, 1.0, 0.0]),
        Vertex::new([0.0, 0.0, 1.0]),
        Vertex::new([1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, 1.0]),
        Vertex::new([0.0, 1.0, 1.0]),
        ], indices: vec!{
            0, 1, 2,
            2, 3, 0,

            1, 5, 6,
            6, 2, 1,

            5, 4, 7,
            7, 6, 5,

            4, 0, 3,
            3, 7, 4,

            3, 2, 6,
            6, 7, 3,

            0, 1, 5,
            5, 4, 0,
        }
    };

    let vertex_buffer = match glium::VertexBuffer::new(&facade, &cube.vertices){
        Ok(buffer) => buffer,
        Err(error) => match error{
            glium::vertex::BufferCreationError::FormatNotSupported => panic!("FormatNotSupported"),
            glium::vertex::BufferCreationError::BufferCreationError(error) => match error{
                glium::buffer::BufferCreationError::OutOfMemory => panic!("OutOfMemory"),
                glium::buffer::BufferCreationError::BufferTypeNotSupported => panic!("BufferTypeNotSupported"),
            }
        },
    };

    let index_buffer = match glium::IndexBuffer::new(&facade, glium::index::PrimitiveType::TrianglesList, &cube.indices) {
        Ok(buffer) => buffer,
        Err(error) => match error {
            glium::index::BufferCreationError::IndexTypeNotSupported => panic!("IndexTypeNotSupported"),
            glium::index::BufferCreationError::PrimitiveTypeNotSupported => panic!("PrimitiveTypeNotSupported"),
            glium::index::BufferCreationError::BufferCreationError(error) => match error{
                glium::buffer::BufferCreationError::OutOfMemory => panic!("OutOfMemory"),
                glium::buffer::BufferCreationError::BufferTypeNotSupported => panic!("BufferTypeNotSupported"),
            }
        },
    };

    let draw_parameters = glium::DrawParameters{
        depth: glium::Depth{
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    let mut keys : Keyboard = Keyboard::new();

    let tps = 60.0;

    let tps_s = 1.0 / tps;

    let mut last_time = time::precise_time_s();
    let mut delta_time = 0.0;

    let mut i = last_time;

    let mut frames = 0;
    let mut ticks = 0;

    loop{
        let now = time::precise_time_s();
        delta_time += now - last_time;
        last_time = now;
        while delta_time > 0.0 {
            //update start
            for event in facade.poll_events() {
                match event {
                    glium::glutin::Event::Resized(x,y) => { //the window was resized
                        resolution = (x, y);
                        mouse_lock_position = update_mouse_lock_position(resolution, mouse_lock_ratio);
                        aspect_ratio = resolution.0 as f32 / resolution.1 as f32;
                        perspective = math::perspective_mat4(
                            near,
                            far,
                            field_of_view,
                            aspect_ratio
                        );
                    }
                    glium::glutin::Event::MouseMoved(coords) => { //the mouse was moved on the window
                        if locked && mouse_lock_position != coords {
                            mouse_delta = (coords.0 as f32 - mouse_lock_position.0 as f32, coords.1 as f32 - mouse_lock_position.1 as f32);
                            camera.yaw += mouse_delta.0 as f32 / d_yaw;
                            camera.pitch += mouse_delta.1 as f32 / d_pitch;
                            println!("({},{}), ({},{})", coords.0, coords.1, mouse_lock_position.0, mouse_lock_position.1);
                            match window.set_cursor_position(mouse_lock_position.0, mouse_lock_position.1){
                                Ok(_) => (),
                                Err(_) => panic!("UnknownError"),
                            };
                        }
                    }
                    glium::glutin::Event::Closed => return,   // the window has been closed by the user
                    glium::glutin::Event::Focused(is_focused) => { // the window gained or lost focus
                        if is_focused {
                            locked = true;
                            match window.set_cursor_state(glium::glutin::CursorState::Grab){
                                Ok(_) => (),
                                Err(string) => panic!(string),
                            };
                        } else {
                            locked = false;
                            match window.set_cursor_state(glium::glutin::CursorState::Normal){
                                Ok(_) => (),
                                Err(string) => panic!(string),
                            };
                        }
                    }
                    glium::glutin::Event::KeyboardInput(state, _, key) => { // a key has been pressed or released
                        match key{
                            Some(key) => keys.set_key_state(key, state),
                            None => (),
                        }
                    }
                    _ => ()
                }
            }
            model = math::translation_mat4([
                x,
                y,
                z,
            ]);
            view = camera.view_mat4();
            uniforms = uniform!{
                model: model,
                view: view,
                perspective: perspective,
            };
            //update end
            delta_time -= tps_s;
            ticks += 1;
        }
        //render start
        let mut frame: Frame = facade.draw();
        let mut color = (0.0, 0.0, 0.0, 1.0);
        match keys.is_key_down(glium::glutin::VirtualKeyCode::Up) {
            glium::glutin::ElementState::Pressed => color.0 = 1.0,
            _ => (),
        };
        match keys.is_key_down(glium::glutin::VirtualKeyCode::Down) {
            glium::glutin::ElementState::Pressed => color.1 = 1.0,
            _ => (),
        };
        match keys.is_key_down(glium::glutin::VirtualKeyCode::Left) {
            glium::glutin::ElementState::Pressed => color.2 = 1.0,
            _ => (),
        };
        frame.clear_color_and_depth(color, 1.0);
        match frame.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_parameters){
            Ok(_) => (),
            Err(error) => panic!(error),
        };
        match frame.finish(){
            Ok(_) => (),
            Err(error) => match error{
                glium::SwapBuffersError::ContextLost => panic!("ContextLost"),
                glium::SwapBuffersError::AlreadySwapped => panic!("AlreadySwapped"),
            },
        };
        //render end
        frames += 1;
		if now > i + 1.0 {
			i += 1.0;
            if debug {
		        println!("{} {}", frames.to_string(), ticks.to_string());
            }
			frames = 0;
			ticks = 0;
		}
    }
}

fn main() {
    let debug = true;
    let x = 0.0;
    let y = 0.0;
    let z = 3.0;
    let near = -10.0;
    let far = 10.0;
    let field_of_view = std::f32::consts::PI / 4.0;
    let d_pitch = -300.0;
    let d_yaw = 300.0;
    run(debug, x, y, z, near, far, field_of_view, d_pitch, d_yaw);
}
