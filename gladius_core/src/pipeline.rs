use std::{io::Write, time::SystemTime};

use crate::prelude::*;
use gladius_shared::prelude::*;
use log::*;

pub trait PipelineCallbacks{
    fn handle_state_update(&mut self, state_message: &str);
    fn handle_settings_error(&mut self, err : SlicerErrors);
    fn handle_settings_warning(&mut self, warning : SlicerWarnings);
    fn handle_commands(&mut self, _moves: &Vec<Command>) {}
    fn handle_calculated_values(&mut self, cv: CalculatedValues, settings: &Settings);
}


pub struct ProfilingCallbacks{
    start_time: SystemTime,
    last_time: SystemTime,
}

impl ProfilingCallbacks{
    pub fn new() -> Self {
        let time = SystemTime::now();
        ProfilingCallbacks {
            start_time: time,
            last_time: time,
        }
    }
}

impl PipelineCallbacks for ProfilingCallbacks{
    fn handle_state_update(&mut self, state_message: &str) {
        let time = SystemTime::now();
        let elapsed = SystemTime::now()
            .duration_since(self.last_time)
            .expect("Time can only go forward");
        self.last_time = time;
        info!("{}\t{}", state_message, elapsed.as_millis());
    }
    
        
    fn handle_settings_error(&mut self, warning : SlicerErrors) {
        let (error_code, message) = warning.get_code_and_message();
        warn!("\n");
        warn!("**************************************************");
        warn!("\tGladius Slicer found a warning");
        warn!("\tWarning Code: {:#X}", error_code);
        warn!("\t{}", message);
        warn!("**************************************************");
        warn!("\n\n\n");
    }
    
    fn handle_settings_warning(&mut self, err : SlicerWarnings) {
        let (error_code, message) = err.get_code_and_message();
        error!("\n");
        error!("**************************************************");
        error!("\tGladius Slicer Ran into an error");
        error!("\tError Code: {:#X}", error_code);
        error!("\t{}", message);
        error!("**************************************************");
        error!("\n\n\n");
    }
    
    fn handle_calculated_values(&mut self, cv: CalculatedValues, settings: &Settings) {
        let (hour, min, sec, _) = cv.get_hours_minutes_seconds_fract_time();
    
        info!(
            "Total Time: {} hours {} minutes {:.3} seconds",
            hour, min, sec
        );
        info!(
            "Total Filament Volume: {:.3} cm^3",
            cv.plastic_volume / 1000.0
        );
        info!("Total Filament Mass: {:.3} grams", cv.plastic_weight);
        info!(
            "Total Filament Length: {:.3} meters",
            cv.plastic_length / 1000.0
        );
        info!(
            "Total Filament Cost: ${:.2}",
            (((cv.plastic_volume / 1000.0) * settings.filament.density) / 1000.0)
                * settings.filament.cost
        );
    }

}


pub fn slicer_pipeline(    models: &[(Vec<Vertex>, Vec<IndexedTriangle>)], settings: &Settings, callbacks: &mut impl PipelineCallbacks, write: &mut impl Write,) -> Result<(),SlicerErrors>{
    handle_setting_validation(settings.validate_settings(), callbacks);

    check_model_bounds(&models, &settings)?;

    callbacks.handle_state_update("Creating Towers");

    let towers: Vec<TriangleTower<_>> = create_towers::<Vertex>( &models)?;

    callbacks.handle_state_update("Slicing");

    let objects = slice(towers, &settings)?;

    callbacks.handle_state_update("Generating Moves");

    let mut moves = generate_moves(objects, &settings, callbacks)?;


    check_moves_bounds(&moves, &settings)?;

    callbacks.handle_state_update("Optimizing");
    debug!("Optimizing {} Moves", moves.len());

    OptimizePass::pass(&mut moves, &settings);
    callbacks.handle_state_update("Slowing Layer Down");

    SlowDownLayerPass::pass(&mut moves, &settings);

    callbacks.handle_commands(&moves);

    callbacks.handle_state_update("Outputting G-code");

    debug!("Converting {} Moves", moves.len());

    convert(&moves, &settings, write)?;

    callbacks.handle_state_update("Calculate Values");
    let cv = calculate_values(&moves, settings);

    callbacks.handle_calculated_values(cv, settings);
    Ok(())


}

fn generate_moves(
    mut objects: Vec<Object>,
    settings: &Settings,
    callbacks: &mut impl PipelineCallbacks, 
) -> Result<Vec<Command>, SlicerErrors> {
    // Creates Support Towers
    SupportTowerPass::pass(&mut objects, settings, callbacks);

    // Adds a skirt
    SkirtPass::pass(&mut objects, settings, callbacks);

    // Adds a brim
    BrimPass::pass(&mut objects, settings, callbacks);

    let v: Result<Vec<()>, SlicerErrors> = objects
        .iter_mut()
        .map(|object| {
            let slices = &mut object.layers;

            // Shrink layer
            ShrinkPass::pass(slices, settings, callbacks)?;

            // Handle Perimeters
            PerimeterPass::pass(slices, settings, callbacks)?;

            // Handle Bridging
            BridgingPass::pass(slices, settings, callbacks)?;

            // Handle Top Layer
            TopLayerPass::pass(slices, settings, callbacks)?;

            // Handle Top And Bottom Layers
            TopAndBottomLayersPass::pass(slices, settings, callbacks)?;

            // Handle Support
            SupportPass::pass(slices, settings, callbacks)?;

            // Lightning Infill
            LightningFillPass::pass(slices, settings, callbacks)?;

            // Fill Remaining areas
            FillAreaPass::pass(slices, settings, callbacks)?;

            // Order the move chains
            OrderPass::pass(slices, settings, callbacks)
        })
        .collect();

    v?;

    Ok(convert_objects_into_moves(objects, settings))
}

/// Sends an apropreate error/warning message for a `SettingsValidationResult`
fn handle_setting_validation(res: SettingsValidationResult, callbacks: &mut impl PipelineCallbacks) {
    match res {
        SettingsValidationResult::NoIssue => {}
        SettingsValidationResult::Warning(slicer_warning) => callbacks.handle_settings_warning(slicer_warning),
        SettingsValidationResult::Error(slicer_error) => {
            callbacks.handle_settings_error(slicer_error);
            std::process::exit(-1);
        }
    }
}