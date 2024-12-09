#![deny(missing_docs)]

use crate::error::SlicerErrors;
use crate::types::{IndexedTriangle, Transform, Vertex};

mod stl;
mod threemf;

pub use stl::STLLoader;
pub use threemf::ThreeMFLoader;

/// The raw triangles and vertices of a model
pub type ModelRawData = (Vec<Vertex>, Vec<IndexedTriangle>);

/// Loader trait to define loading in a file type of a model into a triangles and vertices
pub trait Loader {
    /// Load a specific file
    fn load(&self, filepath: &str) -> Result<Vec<ModelRawData>, SlicerErrors>;
}
