use std::{io::Write, time::SystemTime};

use crate::prelude::*;
use gladius_shared::prelude::*;
use log::{debug, info};

pub trait PipelineCallbacks{
    fn handle_state_update(&mut self, state_message: &str);
    fn handle_commands(&mut self, _moves: &Vec<Command>) {}
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


}


pub fn slicer_pipeline(    models: &[(Vec<Vertex>, Vec<IndexedTriangle>)], settings: &Settings, callbacks: &mut impl PipelineCallbacks, write: &mut impl Write,) -> Result<CalculatedValues,SlicerErrors>{
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
    Ok(calculate_values(&moves, settings))


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
