use anyhow::Result;
use log::debug;
use three_d::*;
use three_d_asset::io::Serialize;

pub fn render_mesh(
    cpu_mesh: &CpuMesh,    // mesh to be rendered
    elevation_angle: f64,  // vertical angles from xy-plane, in degrees
    azimutal_angle: f64,   // horizontal angles from x-axis, in degrees
    target_filename: &str, // filename of the target image (.png preferrably)
    render_size: (u16, u16),
    context: &HeadlessContext,
) -> Result<()> {
    // --- render size ---
    let (width, height) = render_size;
    let width = width as u32;
    let height = height as u32;

    // --- GPU geometry + basic physically based material ---
    debug!("importing mesh");
    let mesh = Mesh::new(&context, &cpu_mesh);
    let material = ColorMaterial::default();
    let model = Gm::new(mesh, material);

    // --- camera ---

    // Distance so the whole mesh fits in the view
    let aabb = model.aabb();
    let center = aabb.center();
    let diag = (aabb.max() - aabb.min()).magnitude();
    let fov = 55.0;
    let distance = diag / (2.0 * ((fov as f32).to_radians() * 0.5).tan());

    let elevation_rad = elevation_angle.to_radians() as f32;
    let azimutal_rad = azimutal_angle.to_radians() as f32;

    let eye = center
        + vec3(
            distance * elevation_rad.cos() * azimutal_rad.cos(),
            distance * elevation_rad.sin(),
            distance * elevation_rad.cos() * azimutal_rad.sin(),
        );

    let viewport = Viewport::new_at_origo(width, height);
    let camera = Camera::new_perspective(
        viewport,
        eye,                 // eye
        center,              // target
        vec3(0.0, 1.0, 0.0), // up vector
        degrees(fov),        // field of view
        10.0,                // near plane
        distance * 10.0,     // far plane safely beyond mesh
    );

    // --- offscreen color + depth textures (render target) ---
    // RGBA8 color
    let mut texture = Texture2D::new_empty::<[u8; 4]>(
        &context,
        width,
        height,
        Interpolation::Linear,
        Interpolation::Linear,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );
    // 32-bit float depth
    let mut depth = DepthTexture2D::new::<f32>(
        &context,
        width,
        height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    debug!("rendering scene");
    let pixels = RenderTarget::new(texture.as_color_target(None), depth.as_depth_target())
        .clear(ClearState::color_and_depth(0.04, 0.04, 0.05, 1.0, 1.0))
        .render(&camera, &model, &[])
        .read_color();

    debug!("exporting to file");
    three_d_asset::io::save(
        &CpuTexture {
            data: TextureData::RgbaU8(pixels),
            width: texture.width(),
            height: texture.height(),
            ..Default::default()
        }
        .serialize(target_filename)?,
    )?;

    Ok(())
}
