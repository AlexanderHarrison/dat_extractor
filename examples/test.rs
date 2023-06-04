use glam::f32::{Quat, Mat4, Vec3};

fn main() {
    let scale: Vec3 = (1., 1., 1.).into();
    let translation: Vec3 = (0., 0., 0.).into();
    let qrot = Quat::from_euler(glam::EulerRot::ZYX, 1.0, 1.0, 1.0);

    let m = Mat4::from_scale_rotation_translation(scale, qrot, translation);
    //let m = Mat4::from_scale(scale);

    println!("{:.2}", m.x_axis.x);
    println!("{:.2}", m.x_axis.y);
    println!("{:.2}", m.x_axis.z);
    println!("{:.2}", m.x_axis.w);
    println!();

    println!("{:.2}", m.y_axis.x);
    println!("{:.2}", m.y_axis.y);
    println!("{:.2}", m.y_axis.z);
    println!("{:.2}", m.y_axis.w);
    println!();

    println!("{:.2}", m.z_axis.x);
    println!("{:.2}", m.z_axis.y);
    println!("{:.2}", m.z_axis.z);
    println!("{:.2}", m.z_axis.w);
    println!();

    println!("{:.2}", m.w_axis.x);
    println!("{:.2}", m.w_axis.y);
    println!("{:.2}", m.w_axis.z);
    println!("{:.2}", m.w_axis.w);
}
