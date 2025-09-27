type AltitudeData = Vec<(i32, i32, f32)>; // x,y,z
type ImageData = Vec<(i32, i32, u8, u8, u8)>; // x,y,r,g,b

mod compose_gif;
mod create_mesh;
mod prepare_data;
mod render_mesh;

pub use compose_gif::compose_gif;
pub use create_mesh::create_mesh;
pub use prepare_data::prepare_data;
pub use render_mesh::render_mesh;
