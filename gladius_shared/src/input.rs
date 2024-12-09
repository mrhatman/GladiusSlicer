use log::{debug, info};

use crate::error::SlicerErrors;
use crate::loader::{Loader, STLLoader, ThreeMFLoader};
use crate::settings::{PartialSettingsFile, Settings};

use crate::types::{IndexedTriangle, InputObject, Transform, Vertex};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Load the models from Input object and return the Vertices and Triangles
pub fn load_models(
    input_objs: Vec<InputObject>,
    settings: &Settings,
) -> Result<Vec<crate::loader::ModelRawData>, SlicerErrors> {
    info!("Loading Input");

    let converted_inputs: Vec<(Vec<Vertex>, Vec<IndexedTriangle>)> =
        input_objs.into_iter().try_fold(vec![], |mut vec, object| {
            let model_path = Path::new(object.get_model_path());

            debug!("Using input file: {:?}", model_path);

            let extension = model_path.extension().and_then(OsStr::to_str).ok_or(
                SlicerErrors::FileFormatNotSupported {
                    filepath: model_path.to_string_lossy().to_string(),
                },
            )?;

            let loader: Result<&dyn Loader, SlicerErrors> = match extension.to_lowercase().as_str()
            {
                "stl" => Ok(&STLLoader),
                "3mf" => Ok(&ThreeMFLoader),
                _ => Err(SlicerErrors::FileFormatNotSupported {
                    filepath: model_path.to_string_lossy().to_string(),
                }),
            };

            info!("Loading model from: {}", &object.get_model_path());

            let models = loader?.load(model_path.to_str().ok_or(SlicerErrors::InputNotUTF8)?)?;

            info!("Loading objects");

            let (x, y) = match object {
                InputObject::AutoTranslate(_, x, y) => (x, y),
                _ => (0.0, 0.0),
            };

            let transform = match object {
                InputObject::Raw(_, transform) => transform,
                InputObject::Auto(_) | InputObject::AutoTranslate(_, _, _) => {
                    let (min_x, max_x, min_y, max_y, min_z) =
                        models.iter().flat_map(|(v, _t)| v.iter()).fold(
                            (
                                f64::INFINITY,
                                f64::NEG_INFINITY,
                                f64::INFINITY,
                                f64::NEG_INFINITY,
                                f64::INFINITY,
                            ),
                            |a, b| {
                                (
                                    a.0.min(b.x),
                                    a.1.max(b.x),
                                    a.2.min(b.y),
                                    a.3.max(b.y),
                                    a.4.min(b.z),
                                )
                            },
                        );
                    Transform::new_translation_transform(
                        (x + settings.print_x - (max_x + min_x)) / 2.,
                        (y + settings.print_y - (max_y + min_y)) / 2.,
                        -min_z,
                    )
                }
            };

            let trans_str =
                serde_json::to_string(&transform).map_err(|_| SlicerErrors::InputMisformat)?;

            debug!("Using Transform {}", trans_str);

            vec.extend(models.into_iter().map(move |(mut v, t)| {
                for vert in &mut v {
                    vert.mul_transform(&transform);
                }

                (v, t)
            }));

            Ok(vec)
        })?;
    Ok(converted_inputs)
}

/// Get the contents from a file or convert error
pub fn load_settings_json(filepath: &str) -> Result<String, SlicerErrors> {
    std::fs::read_to_string(filepath).map_err(|_| SlicerErrors::SettingsFileNotFound {
        filepath: filepath.to_string(),
    })
}

/// Load a settings file from the partial settings json provided at the given filepath
pub fn load_settings(
    filepath: Option<&str>,
    settings_data: &str,
) -> Result<Settings, SlicerErrors> {
    let partial_settings: PartialSettingsFile =
        deser_hjson::from_str(settings_data).map_err(|_| SlicerErrors::SettingsFileMisformat {
            filepath: filepath.unwrap_or("Command Line Argument").to_string(),
        })?;
    let current_path = std::env::current_dir().map_err(|_| SlicerErrors::SettingsFilePermission)?;
    let path = if let Some(fp) = filepath {
        let mut path = PathBuf::from_str(fp).map_err(|_| SlicerErrors::SettingsFileNotFound {
            filepath: fp.to_string(),
        })?;
        path.pop();
        path
    } else {
        current_path
    };

    let settings = partial_settings.get_settings(path)?;

    Ok(settings)
}
