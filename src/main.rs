use clap::{App, load_yaml};
use simple_logger::SimpleLogger;
use log::{LevelFilter};
use crate::types::*;
use crate::loader::*;

use crate::settings::Settings;
use std::io::Write;
use std::fs::File;
use crate::plotter::Slice;
use crate::optimizer::optimize_commands;
use crate::tower::*;
use geo::Coordinate;
use itertools::Itertools;
use geo_clipper::*;


mod loader;
mod types;
mod settings;
mod plotter;
mod optimizer;
mod tower;

fn main() {


    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let settings : Settings = matches.value_of("SETTINGS").map(|str| serde_json::from_str(&std::fs::read_to_string(str).unwrap()).unwrap() ).unwrap_or_default();

        // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    println!("Using input file: {}", matches.value_of("INPUT").unwrap());

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match matches.occurrences_of("verbose") {
        0 => SimpleLogger::new().with_level(LevelFilter::Error ).init().unwrap(),
        1 => SimpleLogger::new().with_level(LevelFilter::Warn ).init().unwrap(),
        2 => SimpleLogger::new().with_level(LevelFilter::Info ).init().unwrap(),
        3 => SimpleLogger::new().with_level(LevelFilter::Debug ).init().unwrap(),
        4 | _ => SimpleLogger::new().with_level(LevelFilter::Trace ).init().unwrap(),
    }

    println!("Loading Input");

    let loader = STLLoader{};
    let (mut vertices,triangles)  =loader.load(matches.value_of("INPUT").unwrap()).unwrap();


    let transform = if let Some(transform_str) = matches.value_of("MANUALTRANFORM") {
        serde_json::from_str(transform_str).unwrap()
    }
    else{
        let (min_x,max_x,min_y,max_y,min_z) = vertices.iter().fold((f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY), |a,b| (a.0.min(b.x),a.1.max(b.x),a.2.min(b.y),a.3.max(b.y),a.4.min(b.z), ));
        Transform::new_translation_transform( (settings.print_x +max_x+min_x) /2.,(settings.print_y+ max_y+min_y) /2.,-min_z)
    };

    let trans_str = serde_json::to_string(&transform).unwrap();

    println!("Using Transform {}",trans_str);

    for vert in vertices.iter_mut(){
        *vert = &transform * *vert;
    }

    println!("Creating Tower");


    let tower = TriangleTower::from_triangles_and_vertices(&triangles,vertices);

    let mut tower_iter = TriangleTowerIterator::new(&tower);

    println!("Slicing");

    let mut moves = vec![];
    let mut layer = 0.0;
    let mut more_lines = true;



    let mut  slices = vec![];

    while more_lines {

        //Advance to the correct height
        if slices.is_empty(){
            //first layer
            layer+= settings.first_layer_height/2.0;
            tower_iter.advance_to_height(layer );
            layer+= settings.first_layer_height/2.0;
        }
        else{
            layer += settings.layer_height/2.0;
            tower_iter.advance_to_height(layer );
            layer += settings.layer_height/2.0;
        }

        //Get the ordered lists of points
        let layer_loops = tower_iter.get_points();

        if layer_loops.is_empty(){
            more_lines = false;
        }
        else {

            //Add this slice to the
            let slice = Slice::from_multiple_point_loop(layer_loops.iter().map(|verts| verts.into_iter().map(|v| Coordinate { x: v.x, y: v.y }).collect::<Vec<Coordinate<f64>>>()).collect());
            slices.push((layer,slice));
        };
    }



    println!("Generating Moves");

    let mut layer_count = 0;

    let slice_count = slices.len();


    //Handle Perimeters
    println!("Generating Moves: Perimeters");
    for (layer,slice) in slices.iter_mut(){
        slice.slice_perimeters_into_chains(&settings);
    }

    //Combine layer to form support

    println!("Generating Moves: Above and below support");

    let layers  = 3;
    for q in layers..slices.len()-layers{

        let below =  slices[(q-layers+1)..q].iter().map(|m| (m.1.get_entire_slice_polygon()))
            .fold(slices.get(q-layers).unwrap().1.get_entire_slice_polygon().clone(),(|a,b| a.intersection(b,10000.0)));
        let above =  slices[q+2..q+layers].iter().map(|m| (m.1.get_entire_slice_polygon()))
            .fold(slices.get(q+1).unwrap().1.get_entire_slice_polygon().clone(),(|a,b| a.intersection(b,10000.0)));
        let intersection  = below.intersection(&above,10000.0);

        slices.get_mut(q).unwrap().1.fill_solid_subtracted_area(&intersection,&settings,q)
    }
    println!("Generating Moves: Fill Areas");
    //Fill all remaining areas
    for (layer,slice) in slices.iter_mut(){
        slice.fill_remaining_area(&settings,layer_count < 3 || layer_count+ 3 +1>slice_count ,layer_count);
        layer_count +=1;
    }
    //Convert all commands into
    println!("Convert into Commnds");
    let mut last_layer = 0.0;
    for (layer,slice) in slices.iter_mut(){
        moves.push(Command::LayerChange {z: *layer});
        slice.slice_into_commands(&settings,&mut moves, *layer - last_layer);

        last_layer = *layer;
    }

    let mut plastic_used = 0.0;
    let mut total_time = 0.0;
    let mut current_speed = 0.0;
    let mut current_pos = Coordinate{x: 0.0,y :0.0};

    for cmd in &moves{

        match cmd {
            Command::MoveTo { end } => {
                let x_diff = end.x-current_pos.x;
                let y_diff = end.y-current_pos.y;
                let d = ((x_diff * x_diff) + (y_diff * y_diff)).sqrt();
                 current_pos = *end;
                if current_speed != 0.0 {
                    total_time += d / current_speed;
                }

            }
            Command::MoveAndExtrude { start, end,width,thickness } => {

                let x_diff = end.x-start.x;
                let y_diff = end.y-start.y;
                let d = ((x_diff * x_diff) + (y_diff * y_diff)).sqrt();
                current_pos = *end;
                total_time += d / current_speed;



                plastic_used += width * thickness *d;
            }
            Command::LayerChange { .. } => {}
            Command::SetState { new_state } => {
                if let Some(speed) = new_state.MovementSpeed{
                    current_speed = speed
                }
            }
            Command::Delay { msec } => {
                total_time += *msec as f64/ 1000.0;
            }
            Command::Arc { .. } => {unimplemented!()}
            Command::NoAction => { }
        }

    }

    println!("Total Time: {} second", total_time);
    println!("Total Filament: {} mm^3",plastic_used );


    //Output the GCode
    if let Some(file_path ) = matches.value_of("OUTPUT"){

        //Output to file
        println!("Optimizing {} Moves", moves.len());
        optimize_commands(&mut moves,&settings);
        println!("Converting {} Moves", moves.len());
        convert(&moves,settings,&mut File::create(file_path).expect("File not Found")).unwrap();
    }
    else{
        //Output to stdout
        println!("Optimizing {} Moves", moves.len());
        let stdout = std::io::stdout();
        optimize_commands(&mut moves,&settings);
        println!("Converting {} Moves", moves.len());
        convert(&moves,settings,&mut stdout.lock()).unwrap();

    };






}


fn convert( cmds: &Vec<Command>, settings: Settings, write:&mut impl Write) ->  Result<(),Box<dyn std::error::Error>>{

    let mut start = settings.starting_gcode.clone();

    start = start.replace("[First Layer Extruder Temp]", &format!("{:.1}",settings.filament.extruder_temp));
    start = start.replace("[First Layer Bed Temp]", &format!("{:.1}",settings.filament.bed_temp));

    writeln!(write,"{}",start)?;

    for cmd in cmds{
        match cmd {
            Command::MoveTo { end,..} => {
                writeln!(write,"G1 X{:.5} Y{:.5}",end.x,end.y )?
            },
            Command::MoveAndExtrude {start,end,width, thickness} => {
                let x_diff = end.x-start.x;
                let y_diff = end.y-start.y;
                let length = ((x_diff * x_diff) + (y_diff * y_diff)).sqrt();

                let extrude = ((4.0 * thickness * width) /(std::f64::consts::PI*settings.filament.diameter*settings.filament.diameter)) *length;

                writeln!(write,"G1 X{:.5} Y{:.5} E{:.5}",end.x ,end.y,extrude)?;
            }
            Command::SetState {new_state} => {

                match new_state.Retract{
                    None => {}
                    Some(dir) => {
                        writeln!(write, "G1 E{} F{} ; Retract or unretract",if dir {-1.0} else {1.0} * settings.retract_length, 60.0 * settings.retract_speed)?;
                    }
                }

                if let Some(speed)  = new_state.MovementSpeed{
                    writeln!(write,"G1 F{:.5}",speed * 60.0 )?;
                }
                if let Some(ext_temp)  = new_state.ExtruderTemp{
                     writeln!(write,"M104 S{:.1} ; set extruder temp",ext_temp )?;
                }
                if let Some(bed_temp)  = new_state.BedTemp{
                     writeln!(write,"M140 S{:.1} ; set bed temp",bed_temp )?;
                }


            }
            Command::LayerChange {z} => {
                writeln!(write,"G1 Z{:.5}",z )?;
            }
            Command::Delay {msec} =>
            {
                writeln!(write,"G4 P{:.5}",msec )?;
            }
            Command::Arc { start,end,center,clockwise,width, thickness} => {
                let x_diff = end.x-start.x;
                let y_diff = end.y-start.y;
                let cord_length = ((x_diff * x_diff) + (y_diff * y_diff)).sqrt();
                let x_diff_r = end.x-center.x;
                let y_diff_r = end.y-center.y;
                let radius = ((x_diff_r * x_diff_r) + (y_diff_r * y_diff_r)).sqrt();

                //Divide the chord length by double the radius.
                let t = cord_length / (2.0*radius);
                //println!("{}",t);
                //Find the inverse sine of the result (in radians).
                //Double the result of the inverse sine to get the central angle in radians.
                let central = t.asin() *2.0;
                //Once you have the central angle in radians, multiply it by the radius to get the arc length.
                let extrusion_length  = central * radius;

                //println!("{}",extrusion_length);
                let extrude = (4.0 * thickness * width*extrusion_length) /(std::f64::consts::PI*settings.filament.diameter*settings.filament.diameter);
                writeln!(write,"{} X{:.5} Y{:.5} I{:.5} J{:.5} E{:.5}",if *clockwise { "G2"} else{"G3"},end.x ,end.y,center.x, center.y, extrude)?;


            }
            Command::NoAction =>{
                panic!("Converter reached a No Action Command, Optimization Failure")
            }
        }
    }

     let end = settings.ending_gcode.clone();

    writeln!(write,"{}",end)?;

    Ok(())

}