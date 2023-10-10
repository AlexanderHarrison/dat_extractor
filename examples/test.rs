use glam::f32::{Quat, Mat4, Vec3, Vec4};

fn main() {
    let m = Mat4::from_scale_rotation_translation(Vec3::splat(2.0), Quat::IDENTITY, 2.0*Vec3::ONE);
    println!("{:?}", m.x_axis);
    println!("{:?}", m.y_axis);
    println!("{:?}", m.z_axis);
    println!("{:?}", m.w_axis);
}
