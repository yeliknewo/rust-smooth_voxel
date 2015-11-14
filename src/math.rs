#![allow(dead_code)]
pub fn magnitude_vec3(vec3: [f32; 3]) -> f32 {
    let mut sum : f32 = 0.0;

    for i in 0..3 {
        sum = sum + vec3[i].powf(2.0);
    }

    sum.sqrt()
}

pub fn scale_vec3(vec3: [f32; 3], scalar: f32) -> [f32; 3] {
    let mut temp = empty_vec3();

    for i in 0..3 {
        temp[i] = vec3[i] * scalar;
    }

    temp
}

pub fn dot_product_vec3(vec_a: [f32; 3], vec_b: [f32; 3]) -> f32 {
    let mut sum : f32 = 0.0;

    for i in 0..3 {
        sum = sum + vec_a[i] * vec_b[i];
    }

    sum
}

pub fn cross_product_vec3(vec_a: [f32; 3], vec_b: [f32; 3]) -> [f32; 3] {
    let mut temp = empty_vec3();

    for i in 0..3 {
        temp[i] = vec_a[(i + 1) % 3] * vec_b[(i + 2) % 3] - vec_b[(i + 1) % 3] * vec_a[(i + 2) % 3];
    }

    temp
}

pub fn empty_vec3() -> [f32; 3] {
    [0.0; 3]
}

pub fn view_matrix_from_radians(pitch: f32, yaw: f32, camera_position: [f32; 3]) -> [[f32; 4]; 4] {
    let pitch_cos = pitch.cos();
    let pitch_sin = pitch.sin();
    let yaw_cos = yaw.cos();
    let yaw_sin = yaw.sin();

    let x = [yaw_cos, 0.0, -yaw_sin];
    let y = [yaw_sin * pitch_sin, pitch_cos, yaw_cos * pitch_sin];
    let z = [yaw_sin * pitch_cos, -pitch_sin, pitch_cos * yaw_cos];

    let mut temp = empty_mat4();

    for i in 0..3 {
        temp[i][0] = x[i];
        temp[i][1] = y[i];
        temp[i][2] = z[i];
    }

    temp[0][3] = -dot_product_vec3(x, camera_position);
    temp[1][3] = -dot_product_vec3(y, camera_position);
    temp[2][3] = -dot_product_vec3(z, camera_position);
    temp[3][3] = 1.0;

    temp
}

pub fn view_mat4_from_vec3(view_direction: [f32; 3], world_up: [f32; 3], camera_position: [f32; 3]) -> [[f32; 4]; 4] {
    let mut temp = empty_mat4();

    let z = scale_vec3(view_direction, -magnitude_vec3(view_direction).recip());

    let dx = cross_product_vec3(view_direction, world_up);

    let x = scale_vec3(dx, magnitude_vec3(dx).recip());

    let y = cross_product_vec3(z, x);

    for i in 0..3 {
        temp[i][0] = x[i];
        temp[i][1] = y[i];
        temp[i][2] = z[i];
        temp[i][3] = camera_position[i];
    }

    temp[0][3] = -dot_product_vec3(x, camera_position);
    temp[1][3] = -dot_product_vec3(y, camera_position);
    temp[2][3] = -dot_product_vec3(z, camera_position);
    temp[3][3] = 1.0;

    temp
}

pub fn perspective_mat4(near: f32, far: f32, field_of_view: f32, aspect_ratio: f32) -> [[f32; 4]; 4] {
    let mut temp = empty_mat4();

    let d = 1.0 / ((field_of_view / 2.0).tan());

    temp[0][0] = d / aspect_ratio;
    temp[1][1] = d;
    temp[2][2] = (near + far) / (near - far);
    temp[2][3] = (2.0 * far) / (near - far);
    temp[3][2] = -1.0;

    temp
}

pub fn translation_mat4(vector: [f32; 3]) -> [[f32; 4]; 4] {
    let mut temp = identity_mat4();

    for y in 0..3 {
        temp[y][3] = vector[y];
    }

    temp
}

pub fn rotation_mat4_x(angle: f32) -> [[f32;  4]; 4] {
    let mut temp = empty_mat4();

    temp[0][0] = 1.0;
    temp[1][1] = angle.cos();
    temp[1][2] = -angle.sin();
    temp[2][1] = angle.sin();
    temp[2][2] = angle.cos();
    temp[3][3] = 1.0;

    temp
}

pub fn rotation_mat4_y(angle: f32) -> [[f32;  4]; 4] {
    let mut temp = empty_mat4();

    temp[0][0] = angle.cos();
    temp[0][2] = angle.sin();
    temp[1][1] = 1.0;
    temp[2][0] = -angle.sin();
    temp[2][2] = angle.cos();
    temp[3][3] = 1.0;

    temp
}

pub fn rotation_mat4_z(angle: f32) -> [[f32;  4]; 4] {
    let mut temp = empty_mat4();

    temp[0][0] = angle.cos();
    temp[0][1] = -angle.sin();
    temp[1][0] = angle.sin();
    temp[1][1] = angle.cos();
    temp[2][2] = 1.0;
    temp[3][3] = 1.0;

    temp
}

pub fn rotation_mat4(vector: [f32; 3]) -> [[f32; 4]; 4] {
    let temp = multiply_mat4_n(vec!(rotation_mat4_x(vector[0]), rotation_mat4_y(vector[1]), rotation_mat4_z(vector[2])));
    temp
}

pub fn multiply_mat4(matrix_a: [[f32; 4]; 4], matrix_b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut temp = empty_mat4();
    for x in 0..4 {
        for y in 0..4 {
            let mut sum = 1.0;
            for e in 0..4 {
                sum = sum + matrix_a[e][x] * matrix_b[y][e];
            }
            temp[y][x] = sum;
        }
    }
    temp
}

pub fn multiply_mat4_n(matrices: Vec<[[f32; 4]; 4]>) -> [[f32; 4]; 4] {
    let mut temp = multiply_mat4(matrices[0], matrices[1]);
    for i in 2..matrices.len() {
        temp = multiply_mat4(temp, matrices[i]);
    }
    temp
}

pub fn empty_mat4() -> [[f32; 4]; 4] {
    [[0.0; 4]; 4]
}

pub fn identity_mat4() -> [[f32; 4]; 4] {
    let mut temp = empty_mat4();
    for i in 0..4 {
        temp[i][i] = 1.0;
    }
    temp
}

pub fn print_mat4(matrix: [[f32; 4]; 4]) {
    for y in 0..4 {
            println!("[{}, {}, {}, {}]", matrix[y][0], matrix[y][1], matrix[y][2], matrix[y][3]);
    }
}
