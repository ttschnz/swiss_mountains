use super::{AltitudeData, ImageData};

use anyhow::Result;
use delaunator::{triangulate, Point};
use kiddo::immutable::float::kdtree::ImmutableKdTree;
use kiddo::SquaredEuclidean;
use log::debug;
use rayon::prelude::*;
use three_d::*;

pub fn create_mesh(altitude_data: AltitudeData, image_data: ImageData) -> Result<CpuMesh> {
    // build kd-tree for color lookup
    debug!("filling kdtree");
    let tree: ImmutableKdTree<f32, u64, 2, 4096> = ImmutableKdTree::new_from_slice(
        &image_data
            .par_iter()
            .map(|(x, z, _r, _g, _b)| [*x as f32, *z as f32])
            .collect::<Vec<_>>(),
    );

    // create color map for lookup
    debug!("filling color map");
    let colors = image_data
        .par_iter()
        .map(|(_x, _z, r, g, b)| Srgba::new_opaque(*r, *g, *b))
        .collect::<Vec<_>>();

    // convert altitude data to vertex positions
    debug!("converting altitude data to vertices");
    let vertices: Vec<Vec3> = altitude_data
        .par_iter()
        .map(|(x, z, y)| vec3(*x as f32, *y, *z as f32))
        .collect();

    // Perform Delaunay triangulation in XZ plane
    debug!("creating triangles");
    let points: Vec<Point> = altitude_data
        .par_iter()
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
    let vertex_colors = altitude_data
        .par_iter()
        .map(|(x, z, _y)| {
            let nearest_color_index = tree.nearest_one::<SquaredEuclidean>(&[*x as f32, *z as f32]);
            colors[nearest_color_index.item as usize]
        })
        .collect::<Vec<_>>();

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
