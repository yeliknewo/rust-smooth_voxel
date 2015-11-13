extern crate num;

use num::traits::{Zero, One, Float};

pub fn mat4_to_f32<T: Float>(matrix: [[T; 4]; 4]) -> [[f32; 4]; 4] {
    let mut temp : [[f32; 4]; 4] = [[0.0; 4]; 4];

    for y in 0..4 {
        for x in 0..4 {
            temp[y][x] = matrix[y][x].to_f32().unwrap();
        }
    }

    temp
}

pub fn perspective_mat4_2<T: Float>(near: T, far: T, field_of_view: T, aspect_ratio: T) -> [[T; 4]; 4] {
    let mut temp = empty_mat4::<T>();

    let d: T = num::cast::<f32, T>(1.0f32).unwrap() / ((field_of_view / num::cast(2.0f32).unwrap()).tan());

    temp[0][0] = d / aspect_ratio;
    temp[1][1] = d;
    temp[2][2] = (near + far) / (near - far);
    temp[2][3] = (num::cast::<f32, T>(2.0f32).unwrap() * far) / (near - far);
    temp[3][2] = num::cast(-1.0f32).unwrap();

    temp
}

pub fn perspective_mat4<T: Float>(near: T, far: T, field_of_view: T, aspect_ratio: T) -> [[T; 4]; 4] {
    let y2 = near * field_of_view.tan();
    let y1 = -y2;
    let x1 = y1 * aspect_ratio;
    let x2 = y2 * aspect_ratio;
    let temp: [[T; 4]; 4] = frustrum_mat4(x1, x2, y1, y2, near, far);
    temp
}

pub fn frustrum_mat4<T: Float>(left: T, right: T, bottom: T, top: T, near: T, far: T) -> [[T; 4]; 4] {
    let mut temp = empty_mat4::<T>();

    temp[0][0] = (near * num::cast(2.0f32).unwrap()) / (right - left);
    temp[1][1] = (near * num::cast(2.0f32).unwrap()) / (top - bottom);
    temp[2][2] = (near + far) / (near - far);
    temp[3][3] = num::cast(0.0f32).unwrap();

    temp[0][2] = (right + left) / (right - left);
    temp[1][2] = (top + bottom) / (top - bottom);
    temp[3][2] = num::cast(-1.0f32).unwrap();
    temp[2][3] = (far * near * num::cast(-2.0f32).unwrap()) / (far / near);

    temp
}

pub fn translation_mat4<T: Float>(vector: [T; 3]) -> [[T; 4]; 4] {
    let mut temp = identity_mat4::<T>();

    for y in 0..3 {
        temp[y][3] = vector[y];
    }

    temp
}

pub fn rotation_mat4_x<T: Float>(angle: T) -> [[T;  4]; 4] {
    let mut temp = empty_mat4::<T>();

    temp[0][0] = One::one();
    temp[1][1] = angle.cos();
    temp[1][2] = -angle.sin();
    temp[2][1] = angle.sin();
    temp[2][2] = angle.cos();
    temp[3][3] = One::one();

    temp
}

pub fn rotation_mat4_y<T: Float>(angle: T) -> [[T;  4]; 4] {
    let mut temp = empty_mat4::<T>();

    temp[0][0] = angle.cos();
    temp[0][2] = angle.sin();
    temp[1][1] = One::one();
    temp[2][0] = -angle.sin();
    temp[2][2] = angle.cos();
    temp[3][3] = One::one();

    temp
}

pub fn rotation_mat4_z<T: Float>(angle: T) -> [[T;  4]; 4] {
    let mut temp = empty_mat4::<T>();

    temp[0][0] = angle.cos();
    temp[0][1] = -angle.sin();
    temp[1][0] = angle.sin();
    temp[1][1] = angle.cos();
    temp[2][2] = One::one();
    temp[3][3] = One::one();

    temp
}

pub fn rotation_mat4<T: Float>(vector: [T; 3]) -> [[T; 4]; 4] {
    let temp = multiply_mat4_n::<T>(vec!(rotation_mat4_x::<T>(vector[0]), rotation_mat4_y::<T>(vector[1]), rotation_mat4_z::<T>(vector[2])));
    temp
}

pub fn multiply_mat4<T: Float>(matrix_a: [[T; 4]; 4], matrix_b: [[T; 4]; 4]) -> [[T; 4]; 4] {
    let mut temp = empty_mat4::<T>();
    for x in 0..4 {
        for y in 0..4 {
            let mut sum: T = Zero::zero();
            for e in 0..4 {
                sum = sum + matrix_a[e][x] * matrix_b[y][e];
            }
            temp[y][x] = sum;
        }
    }
    temp
}

pub fn multiply_mat4_n<T: Float>(matrices: Vec<[[T; 4]; 4]>) -> [[T; 4]; 4] {
    let mut temp = multiply_mat4::<T>(matrices[0], matrices[1]);
    for i in 2..matrices.len() {
        temp = multiply_mat4::<T>(temp, matrices[i]);
    }
    temp
}

pub fn empty_mat4<T: Float>() -> [[T; 4]; 4] {
    let temp :[[T; 4]; 4] = [[Zero::zero(); 4]; 4];
    temp
}

pub fn identity_mat4<T: Float>() -> [[T; 4]; 4] {
    let mut temp = empty_mat4::<T>();
    for i in 0..4 {
        temp[i][i] = One::one();
    }
    temp
}

pub fn print_mat4<T: Float>(matrix: [[T; 4]; 4]) {
    for y in 0..4 {
            println!("[{}, {}, {}, {}]", matrix[y][0].to_f32().unwrap(), matrix[y][1].to_f32().unwrap(), matrix[y][2].to_f32().unwrap(), matrix[y][3].to_f32().unwrap());
    }
}
