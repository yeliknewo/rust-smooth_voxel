#[macro_use]
extern crate glium;
extern crate clock_ticks;
extern crate num;

mod math;

use glium::{DisplayBuild, Surface, Frame};
use glium::backend::glutin_backend::GlutinFacade;

use num::traits::{Float};

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

fn run<T: Float>(dx: T, dy: T, dz: T, x: T, y: T, z: T, count_start: T, count_step: T, near: T, far: T, field_of_view: T, aspect_ratio: T){
    let mut resolution : (u32, u32) = (1920, 1280);
    let mut mouse_pos : (i32, i32) = (0,0);

    let facade: GlutinFacade = glium::glutin::WindowBuilder::new()
        .with_title("Smooth Voxel".to_string())
        .with_dimensions(resolution.0, resolution.1)
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;

        uniform mat4 camera;
        uniform mat4 transform;
        uniform mat4 perspective;

        out vec3 o_pos;

        void main() {
            o_pos = position;
            gl_Position = vec4(position, 1.0) * camera * transform * perspective;
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

    let perspective = math::mat4_to_f32::<T>(
        math::perspective_mat4_2::<T>(
            near,
            far,
            field_of_view,
            aspect_ratio
        )
    );

    math::print_mat4::<f32>(perspective);

    let mut transform = math::mat4_to_f32::<T>(
    math::translation_mat4::<T>([
            x,
            y,
            z,
        ])
    );

    math::print_mat4::<f32>(transform);

    let mut uniforms = uniform!{
        camera: math::mat4_to_f32::<T>(math::identity_mat4::<T>()),
        transform: transform,
        perspective: perspective,
    };

    let cube : Cube<u32> = Cube{vertices: [
        Vertex::new([0.0, 0.0, 0.0]),
        Vertex::new([0.5, 0.0, 0.0]),
        Vertex::new([0.5, 0.5, 0.0]),
        Vertex::new([0.0, 0.5, 0.0]),
        Vertex::new([0.0, 0.0, 0.5]),
        Vertex::new([0.5, 0.0, 0.5]),
        Vertex::new([0.5, 0.5, 0.5]),
        Vertex::new([0.0, 0.5, 0.5]),
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
    }};

    let vertex_buffer = glium::VertexBuffer::new(&facade, &cube.vertices).unwrap();

    let index_buffer = glium::IndexBuffer::new(&facade, glium::index::PrimitiveType::TrianglesList, &cube.indices).unwrap();

    let draw_parameters = glium::DrawParameters{
        depth: glium::Depth{
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    let tps = 60.0;

    let tps_s = 1.0 / tps;

    let mut last_time = clock_ticks::precise_time_s();
    let mut delta_time = 0.0;

    let mut i = last_time;

    let mut frames = 0;
    let mut ticks = 0;

    let mut count : T = count_start;

    loop{
        let now = clock_ticks::precise_time_s();
        delta_time += now - last_time;
        last_time = now;
        while delta_time > 0.0 {
            for ev in facade.poll_events() {
                match ev {
                    glium::glutin::Event::Resized(x,y) => resolution = (x, y),
                    glium::glutin::Event::MouseMoved(coords) => mouse_pos = (coords.0, coords.1),
                    glium::glutin::Event::Closed => return,   // the window has been closed by the user
                    _ => ()
                }
            }
            transform = math::mat4_to_f32::<T>(
                math::translation_mat4::<T>([
                    x,
                    y,
                    z,
                ])
            );
            //math::print_mat4::<f32>(transform);
            uniforms = uniform!{
                camera: math::mat4_to_f32::<T>(
                    math::rotation_mat4::<T>([
                        num::cast::<i32, T>(mouse_pos.0).unwrap() / num::cast::<u32, T>(resolution.0).unwrap() * num::cast::<f64, T>(std::f64::consts::PI).unwrap(),
                        num::cast::<i32, T>(mouse_pos.1).unwrap() / num::cast::<u32, T>(resolution.1).unwrap() * num::cast::<f64, T>(std::f64::consts::PI).unwrap(),
                        count / dz,
                    ])
                ),
                transform: transform,
                perspective: perspective,
            };
            delta_time -= tps_s;
            ticks += 1;
            count = count + count_step;
        }
        let mut frame: Frame = facade.draw();
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        frame.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_parameters).unwrap();
        frame.finish().unwrap();
        frames += 1;
		if now > i + 1.0 {
			i += 1.0;
			println!("{} {}", frames.to_string(), ticks.to_string());
			frames = 0;
			ticks = 0;
		}
    }
}

fn main() {
    run::<f32>(1000.0, 2000.0, 3000.0, 0.0, 0.0, -2.0, 0.0, 1.0, -10.0, 10.0, std::f32::consts::PI / 2.0, 16.0 / 9.0);
}
