#![deny(clippy::unwrap_used)]
#![warn(clippy::all, clippy::perf, clippy::missing_const_for_fn)]

use clap::Parser;

use gladius_core::{pipeline::slicer_pipeline, prelude::*};
use gladius_shared::prelude::*;



use std::{fs::File, io::Write};


use log::{debug, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::io::BufWriter;

mod test;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    clap::ArgGroup::new("settings-group")
        .required(true)
        .args(&["settings_file_path", "settings_json"]),
))]
struct Args {
    #[arg(
        required = true,
        help = "The input files and there translations.\nBy default it takes a list of json strings that represents how the models should be loaded and translated.\nSee simple_input for an alterantive command. "
    )]
    input: Vec<String>,

    #[arg(short = 'o', help = "Sets the output dir")]
    output: Option<String>,

    #[arg(short = 'v', action = clap::ArgAction::Count, conflicts_with = "message", help = "Sets the level of verbosity")]
    verbose: u8,

    #[arg(short = 's', help = "Sets the settings file to use")]
    settings_file_path: Option<String>,
    #[arg(short = 'S', help = "The contents of a json settings file.")]
    settings_json: Option<String>,
    #[arg(short = 'm', help = "Use the Message System (useful for interprocess communication)")]
    message: bool,

    #[arg(
        long = "print_settings",
        help = "Print the final combined settings out to Stdout and Terminate. Verbose level 4 will print but continue."
    )]
    print_settings: bool,
    #[arg(
        long = "simple_input",
        help = "The input should only be a list of files that will be auto translated to the center of the build plate."
    )]
    simple_input: bool,
    #[arg(
        short = 'j',
        help = "Sets the number of threads to use in the thread pool (defaults to number of CPUs)"
    )]
    thread_count: Option<usize>,
}

fn main() {
    #[cfg(feature = "json_schema_gen")]
    // export json schema for settings
    Settings::gen_schema(Path::new("settings/")).expect("The programme should exit if this fails");

    // The YAML file is found relative to the current file, similar to how modules are found
    let args: Args = Args::parse();

    // set number of cores for rayon
    if let Some(number_of_threads) = args.thread_count {
        rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_threads)
            .build_global()
            .expect("Only call to build global");
    }

    let mut state_context = StateContext::new(if args.message {
        DisplayType::Message
    } else {
        DisplayType::StdOut
    });

    if !args.message {
        // Vary the output based on how many times the user used the "verbose" flag
        // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'

        SimpleLogger::new()
            .with_level(match args.verbose {
                0 => LevelFilter::Error,
                1 => LevelFilter::Warn,
                2 => LevelFilter::Info,
                3 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            })
            .init()
            .expect("Only Logger Setup");
    }

    //state_update("Loading Inputs", &mut state_context);

    let settings_json = args.settings_json.unwrap_or_else(|| {
        handle_err_or_return(
            input::load_settings_json(
                args.settings_file_path
                    .as_deref()
                    .expect("CLAP should handle requring a settings option to be Some"),
            ),
            &state_context,
        )
    });

    let settings = handle_err_or_return(
        load_settings(args.settings_file_path.as_deref(), &settings_json),
        &state_context,
    );

    let input_objs :Vec<InputObject> = handle_err_or_return(args.input.iter().map(|value|{
        if args.simple_input {
            Ok(InputObject::Auto(value.clone()))
        } else {
            deser_hjson::from_str(&value).map_err(|_| SlicerErrors::InputMisformat)
        }
    }).collect(),&state_context);



    let models = handle_err_or_return(
        crate::input::load_models( input_objs, &settings),
        &state_context,
    );
    if args.print_settings {
        for line in gladius_shared::settings::SettingsPrint::to_strings(&settings) {
            println!("{}", line);
        }
        std::process::exit(0);
    } else if log::log_enabled!(log::Level::Trace) {
        for line in gladius_shared::settings::SettingsPrint::to_strings(&settings) {
            log::trace!("{}", line);
        }
    }

    handle_setting_validation(settings.validate_settings(), &state_context);

    // Output the GCode
    let cv = if let Some(file_path) = &args.output {
        // Output to file
        let mut file = handle_err_or_return(
            File::create(file_path).map_err(|_| SlicerErrors::FileCreateError {
                filepath: file_path.to_string(),
            }),
            &state_context
        );

        let mut profiling_callbacks = ProfilingCallbacks::new();

        handle_err_or_return( 
            slicer_pipeline(
                &models,
                &settings,
                &mut profiling_callbacks,
                &mut file
            ),
            &state_context
        
        )
    } else {
        match state_context.display_type {
            DisplayType::Message => {
                // Output as message
                let mut gcode: Vec<u8> = Vec::new();
                let mut messaging_callbacks = MessageCallbacks{};

                let cv =handle_err_or_return( 
                    slicer_pipeline(
                        &models,
                        &settings,
                        &mut messaging_callbacks,
                        &mut gcode
                        ),
                    &state_context
                
                );
                let message = Message::GCode(
                    String::from_utf8(gcode)
                        .expect("All write occur from write macro so should be utf8"),
                );
                bincode::serialize_into(BufWriter::new(std::io::stdout()), &message)
                    .expect("Write Limit should not be hit");

                cv
            }
            DisplayType::StdOut => {
                // Output to stdout
                let stdout = std::io::stdout();
                let mut stdio_lock = stdout.lock();
                let mut profiling_callbacks = ProfilingCallbacks::new();

                
                handle_err_or_return( 
                    slicer_pipeline(
                        &models,
                        &settings,
                        &mut profiling_callbacks,
                        &mut stdio_lock
                        ),
                    &state_context
                
                )
            }
        }
    };

    print_info_message(cv, &settings, &state_context);


    if let DisplayType::StdOut = state_context.display_type {
        info!(
            "Total slice time {} msec",
            state_context.get_total_elapsed_time().as_millis()
        );
    }
}

/// Display info about the print; time and filament info
fn print_info_message( cv: CalculatedValues, settings: &Settings, state_context: &StateContext) {

    match state_context.display_type{
        DisplayType::Message => {
            let message = Message::CalculatedValues(cv);
            bincode::serialize_into(BufWriter::new(std::io::stdout()), &message)
                .expect("Write Limit should not be hit");
        },
        DisplayType::StdOut => {
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
        },
    }

}


fn handle_err_or_return<T>(res: Result<T, SlicerErrors>, state_context: &StateContext) -> T {
    match res {
        Ok(data) => data,
        Err(slicer_error) => {
            match state_context.display_type {
                DisplayType::Message => send_error_message(slicer_error),
                DisplayType::StdOut => show_error_message(&slicer_error),
            }
            std::process::exit(-1);
        }
    }
}

/// Sends an apropreate error/warning message for a `SettingsValidationResult`
fn handle_setting_validation(res: SettingsValidationResult, state_context: &StateContext) {
    match res {
        SettingsValidationResult::NoIssue => {}
        SettingsValidationResult::Warning(slicer_warning) => match state_context.display_type {
            DisplayType::Message => send_warning_message(slicer_warning),
            DisplayType::StdOut => show_warning_message(&slicer_warning),
        },
        SettingsValidationResult::Error(slicer_error) => {
            match state_context.display_type {
                DisplayType::Message => send_error_message(slicer_error),
                DisplayType::StdOut => show_error_message(&slicer_error),
            }
            std::process::exit(-1);
        }
    }
}

pub struct MessageCallbacks{
}


impl PipelineCallbacks for MessageCallbacks{
    fn handle_state_update(&mut self, state_message: &str) {
        let stdout = std::io::stdout();
        let mut stdio_lock = stdout.lock();
        let message = Message::StateUpdate(state_message.to_string());
        bincode::serialize_into(&mut stdio_lock, &message)
            .expect("Write Limit should not be hit");
        stdio_lock.flush().expect("Standard Out should be limited");
    }
    
    fn handle_commands(&mut self, moves: &Vec<Command>) {
        let message = Message::Commands(moves.clone());
        bincode::serialize_into(BufWriter::new(std::io::stdout()), &message)
            .expect("Write Limit should not be hit");
    }

}
