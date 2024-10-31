use crate::{Object, Settings, Slice, SlicerErrors, TriangleTower, TriangleTowerIterator, Vertex};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use geo::geometry::Coord;

pub fn slice(towers: &[TriangleTower], settings: &Settings) -> Result<Vec<Object>, SlicerErrors> {
    towers
        .par_iter()
        .map(|tower| {
            let mut tower_iter = TriangleTowerIterator::new(tower);

            let mut layer = 0.0;

            let res_points: Result<Vec<(f64, f64, Vec<Vec<Vertex>>)>, SlicerErrors> =
                std::iter::repeat(())
                    .enumerate()
                    .map(|(layer_count, ())| {
                        // Advance to the correct height
                        let layer_height = settings
                            .get_layer_settings(
                                layer_count as u32, // I doute your layer_count will go past 4,294,967,295
                                layer,
                            )
                            .layer_height;

                        let bottom_height = layer;
                        layer += layer_height / 2.0;
                        tower_iter.advance_to_height(layer)?;
                        layer += layer_height / 2.0;

                        let top_height = layer;

                        // Get the ordered lists of points
                        Ok((bottom_height, top_height, tower_iter.get_points()))
                    })
                    .take_while(|r| {
                        if let Ok((_, _, layer_loops)) = r {
                            !layer_loops.is_empty()
                        } else {
                            true
                        }
                    })
                    .collect();

            let points = res_points?;

            let slices: Result<Vec<Slice>, SlicerErrors> = points
                .par_iter()
                .enumerate()
                .map(|(count, (bot, top, layer_loops))| {
                    // Add this slice to the
                    Slice::from_multiple_point_loop(
                        layer_loops
                            .iter()
                            .map(|verts| {
                                verts
                                    .iter()
                                    .map(|v| Coord { x: v.x, y: v.y })
                                    .collect::<Vec<Coord<f64>>>()
                            })
                            .collect(),
                        *bot,
                        *top,
                        count as u32, // I doute your layer_count will go past 4,294,967,295,
                        settings,
                    )
                })
                .collect();

            Ok(Object { layers: slices? })
        })
        .collect()
}
