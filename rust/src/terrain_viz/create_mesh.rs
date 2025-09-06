use super::{AltitudeData, ImageData};

use anyhow::Result;
use delaunator::{triangulate, Point};
use kiddo::float::kdtree::KdTree;
use kiddo::SquaredEuclidean;
use log::debug;
use three_d::*;

pub fn create_mesh(altitude_data: AltitudeData, image_data: ImageData) -> Result<CpuMesh> {
    // build kd-tree for color lookup
    // create color map for lookup
    let mut tree: KdTree<f32, u64, 2, 1024, u32> = KdTree::with_capacity(image_data.len());

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
