use crate::{swissalti3d, swissimage, utils::bounding_box::BoundingBox};
//use nalgebra::DelaunayTriangulation;
use anyhow::Result;
use delaunator::{triangulate, Point};
use kiddo::float::kdtree::KdTree;
use kiddo::SquaredEuclidean;
use log::debug;
use std::{cmp, error::Error};
use three_d::*;
use three_d_asset::io::Serialize;

type AltitudeData = Vec<(i32, i32, f32)>; // x,y,z
type ImageData = Vec<(i32, i32, u8, u8, u8)>; // x,y,r,g,b

pub fn prepare_data(
    peak_coordinates: (u32, u32),
    radius: u32,
    width: u16,
    offline: bool,
) -> Result<(AltitudeData, ImageData), Box<dyn Error>> {
    let peak_north = cmp::min(peak_coordinates.0, peak_coordinates.1) as i32;
    let peak_east = cmp::max(peak_coordinates.0, peak_coordinates.1) as i32;

    let bounding_box = BoundingBox::from_ranges(
        (peak_east - radius as i32, peak_east + radius as i32),
        (peak_north - radius as i32, peak_north + radius as i32),
    );
    let total_width = 2 * radius;
    let step = total_width as usize / width as usize;

    if !offline {
        // cache altitude points
        let swissalti3d_url_list = swissalti3d::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} height files", swissalti3d_url_list.len());
        for url in swissalti3d_url_list {
            swissalti3d::fetch::prefetch(url)?;
        }

        // cache colors
        let swissimage_url_list = swissimage::fetch::get_url_list(&bounding_box)?;
        debug!("prefetching {} color files", swissimage_url_list.len());
        for url in swissimage_url_list {
            swissimage::fetch::prefetch(url)?;
        }
    }

    debug!("collecting altitude points from cache");
    let complete_altitude_data = swissalti3d::cache::get_from_cache(step, &bounding_box)?;
    let altitude_data = complete_altitude_data
        .iter()
        .filter(|(x, y, _z)| {
            // make the data a circle around the peak
            (x - peak_east).pow(2) + (y - peak_north).pow(2) < radius.pow(2) as i32
        })
        .cloned()
        .collect::<AltitudeData>();

    debug!("collecting colors from cache");
    let image_data = swissimage::cache::get_from_cache(step, &bounding_box)?;

    Ok((altitude_data, image_data))
}

pub fn create_mesh(altitude_data: AltitudeData, image_data: ImageData) -> Result<CpuMesh> {
    // build kd-tree for color lookup
    // create color map for lookup
    let mut tree: KdTree<f32, u64, 2, 512, u32> = KdTree::with_capacity(image_data.len());

    debug!("filling kdtree");
    let mut colors: Vec<Srgba> = Vec::new();
    for (i, (x, z, r, g, b)) in image_data.iter().enumerate() {
        tree.add(&[*x as f32, *z as f32], i as u64);
        colors.push(Srgba::new_opaque(*r, *g, *b));
    }

    // convert altitude data to vertex positions
    debug!("converting altitude data to vertices");
    let vertices: Vec<Vec3> = altitude_data
        .iter()
        .map(|(x, z, y)| vec3(*x as f32, *y, *z as f32))
        .collect();

    // Perform Delaunay triangulation in XZ plane
    debug!("creating triangles");
    let points: Vec<Point> = altitude_data
        .iter()
        .map(|(x, z, _)| Point {
            x: *x as f64,
            y: *z as f64,
        })
        .collect();
    let triangulation = triangulate(&points);

    // Collect triangle indices
    let indices: Vec<[u32; 3]> = triangulation
        .triangles
        .chunks_exact(3)
        .map(|tri| [tri[0] as u32, tri[1] as u32, tri[2] as u32])
        .collect();

    // find closest points to each corner (vertex)
    // as the grids don't necessarily align, we find the closest image point in the xz-plane
    debug!("coloring mesh");
    let mut vertex_colors = Vec::new();
    for (x, z, _y) in altitude_data.iter() {
        let nearest_color_index = tree.nearest_one::<SquaredEuclidean>(&[*x as f32, *z as f32]);
        let color: Srgba = colors[nearest_color_index.item as usize];
        vertex_colors.push(color);
    }

    // build CPU mesh
    debug!("building mesh");
    let mut cpu_mesh = CpuMesh {
        positions: Positions::F32(vertices.clone()),
        indices: Indices::U32(indices.clone().into_iter().flatten().collect()),
        colors: Some(vertex_colors),
        ..Default::default()
    };

    debug!("computing normals");
    cpu_mesh.compute_normals();

    Ok(cpu_mesh)
}

pub fn render_mesh(
    cpu_mesh: &CpuMesh,    // mesh to be rendered
    elevation_angle: f64,  // vertical angles from xy-plane, in degrees
    azimutal_angle: f64,   // horizontal angles from x-axis, in degrees
    target_filename: &str, // filename of the target image (.png preferrably)
    context: &HeadlessContext,
) -> Result<()> {
    // --- render size ---
    let (width, height) = (1024u32, 1024u32);

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
        0.01,                // near plane
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
