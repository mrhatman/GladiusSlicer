#![deny(missing_docs)]
use std::{io::Write, time::{Duration, SystemTime}};

use crate::{bounds_checking::{check_model_bounds, check_moves_bounds}, command_pass::{CommandPass, OptimizePass, SlowDownLayerPass}, converter::convert, plotter::convert_objects_into_moves, prelude::*, slice_pass::*, slicing::slice, tower::{create_towers, TriangleTower}};
use gladius_shared::prelude::*;
use log::*;

///Callbacks for the slicer pipeline to allow calls to control what happens during the process
pub trait PipelineCallbacks{
    /// Called whenever the current state changes
    /// State message refelects the new state of the slicing process
    /// Useful for telling user what the slicer is working on 
    fn handle_state_update(&mut self, state_message: &str);
    
    /// Handle the case of a warning being found in settings validation
    /// Warning will not stop the slcing process by default
    fn handle_settings_warning(&mut self, warning : SlicerWarnings);

    /// Callback for the Final commands after optomization
    fn handle_commands(&mut self, _moves: &Vec<Command>) {}

    /// Callback for the calculated values
    fn handle_calculated_values(&mut self, cv: CalculatedValues, settings: &Settings);
}


///A basic set of that logs most messages and profiles based on state callbacks 
pub struct ProfilingCallbacks{
    start_time: SystemTime,
    last_time: SystemTime,
}

impl ProfilingCallbacks{
    /// Create a new Set of callbacks
    /// Starts the time for total elapsed time 
    pub fn new() -> Self {
        let time = SystemTime::now();
        ProfilingCallbacks {
            start_time: time,
            last_time: time,
        }
    }

    ///Gets the total system time since the new call   
    pub fn get_total_elapsed_time(&self) -> Duration {
        let elapsed = SystemTime::now()
            .duration_since(self.start_time)
            .expect("Time can only go forward");
        elapsed
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
    
    
    
    fn handle_settings_warning(&mut self, warn : SlicerWarnings) {
        let (error_code, message) = warn.get_code_and_message();
        warn!("\n");
        warn!("**************************************************");
        warn!("\tGladius Slicer found a warning");
        warn!("\tWarning Code: {:#X}", error_code);
        warn!("\t{}", message);
        warn!("**************************************************");
        warn!("\n\n\n");
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

/// The primary pipeline for slicing
pub fn slicer_pipeline(    models: &[(Vec<Vertex>, Vec<IndexedTriangle>)], settings: &Settings, callbacks: &mut impl PipelineCallbacks, write: &mut impl Write,) -> Result<(),SlicerErrors>{
    handle_setting_validation(settings.validate_settings(), callbacks)?;

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
fn handle_setting_validation(res: SettingsValidationResult, callbacks: &mut impl PipelineCallbacks) -> Result<(), SlicerErrors> {
    match res {
        SettingsValidationResult::NoIssue => {
            Ok(())
        }
        SettingsValidationResult::Warning(slicer_warning) => {
            callbacks.handle_settings_warning(slicer_warning);
            Ok(())
        },
        SettingsValidationResult::Error(slicer_error) => {
            Err(slicer_error)
        }
    }
}