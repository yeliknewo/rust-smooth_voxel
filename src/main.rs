#[macro_use]
extern crate glium;
extern crate clock_ticks;
extern crate num;

mod math;

use glium::{DisplayBuild, Surface, Frame};
use glium::backend::glutin_backend::GlutinFacade;
use std::collections::HashMap;

struct Camera{
    position: [f32; 3],
    pitch: f32,
    yaw: f32,
}

struct Keyboard {
    keys: HashMap<u8, bool>,
}

impl Keyboard {
    pub fn key_down(&self, key_code: u8) -> bool {
        *self.keys.get(&key_code).unwrap()
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

fn run(debug: bool, x: f32, y: f32, z: f32, near: f32, far: f32, field_of_view: f32, d_pitch: f32, d_yaw: f32){
    let mut resolution : (u32, u32) = (1920, 1280);
    let mut mouse_delta : (f32, f32);
    let mut aspect_ratio = resolution.0 as f32 / resolution.1 as f32;

    let mouse_lock_ratio : (f32, f32) = (0.5, 0.5);
    let mut mouse_lock_position : (f32, f32) = (resolution.0 as f32 * mouse_lock_ratio.0, resolution.1 as f32 * mouse_lock_ratio.1);

    let facade: GlutinFacade = glium::glutin::WindowBuilder::new()
        .with_title("Smooth Voxel".to_string())
        .with_fullscreen(glium::glutin::get_primary_monitor())
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let mut locked = true;

    let window = facade.get_window().unwrap();

    window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();

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

    let program = glium::Program::from_source(&facade, vertex_shader_src, fragment_shader_src, None).unwrap();

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

    let vertex_buffer = glium::VertexBuffer::new(&facade, &cube.vertices).unwrap();

    let index_buffer = glium::IndexBuffer::new(&facade, glium::index::PrimitiveType::TrianglesList, &cube.indices).unwrap();

    let draw_parameters = glium::DrawParameters{
        depth: glium::Depth{
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    let tps = 60.0;

    let tps_s = 1.0 / tps;

    let mut last_time = clock_ticks::precise_time_s();
    let mut delta_time = 0.0;

    let mut i = last_time;

    let mut frames = 0;
    let mut ticks = 0;

    loop{
        let now = clock_ticks::precise_time_s();
        delta_time += now - last_time;
        last_time = now;
        while delta_time > 0.0 {
            for event in facade.poll_events() {
                match event {
                    glium::glutin::Event::Resized(x,y) => {
                        resolution = (x, y);
                        mouse_lock_position = (resolution.0 as f32 / mouse_lock_ratio.0, resolution.1 as f32 / mouse_lock_ratio.1);
                        aspect_ratio = resolution.0 as f32 / resolution.1 as f32;
                        perspective = math::perspective_mat4(
                            near,
                            far,
                            field_of_view,
                            aspect_ratio
                        );
                    }
                    glium::glutin::Event::MouseMoved(coords) => {
                        if locked {
                            mouse_delta = (coords.0 as f32 - mouse_lock_position.0, coords.1 as f32 - mouse_lock_position.1);
                            camera.yaw += mouse_delta.0 as f32 / d_yaw;
                            camera.pitch -= mouse_delta.1 as f32 / d_pitch;
                            window.set_cursor_position(mouse_lock_position.0 as i32, mouse_lock_position.1 as i32).unwrap();
                        }
                    }
                    glium::glutin::Event::Closed => return,   // the window has been closed by the user
                    glium::glutin::Event::Focused(is_focused) => {
                        if is_focused {
                            locked = true;
                            window.set_cursor_state(glium::glutin::CursorState::Grab).unwrap();
                        } else {
                            locked = false;
                            window.set_cursor_state(glium::glutin::CursorState::Normal).unwrap();
                        }
                    }
                    glium::glutin::Event::KeyboardInput(state, id, key) => {
                        match state {
                            glium::glutin::ElementState::Pressed => {
                                match key.unwrap() {
                                    glium::glutin::VirtualKeyCode::Up => {

                                    }
                                    glium::glutin::VirtualKeyCode::Down => {

                                    }
                                    glium::glutin::VirtualKeyCode::Left => {

                                    }
                                    glium::glutin::VirtualKeyCode::Right => {

                                    }
                                    _ => ()
                                }
                            }
                            glium::glutin::ElementState::Released => {
                                match key.unwrap() {
                                    glium::glutin::VirtualKeyCode::Up => {

                                    }
                                    glium::glutin::VirtualKeyCode::Down => {

                                    }
                                    glium::glutin::VirtualKeyCode::Left => {

                                    }
                                    glium::glutin::VirtualKeyCode::Right => {

                                    }
                                    _ => ()
                                }
                            }
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
            delta_time -= tps_s;
            ticks += 1;
        }
        let mut frame: Frame = facade.draw();
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        frame.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_parameters).unwrap();
        frame.finish().unwrap();
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
    let d_pitch = 300.0;
    let d_yaw = 300.0;
    run(debug, x, y, z, near, far, field_of_view, d_pitch, d_yaw);
}
