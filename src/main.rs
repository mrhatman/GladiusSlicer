#![deny(clippy::unwrap_used)]
#![warn(clippy::all, clippy::perf, clippy::missing_const_for_fn)]
use clap::Parser;

use gladius_core::{pipeline::slicer_pipeline, prelude::*};
use gladius_shared::prelude::*;

use std::{fs::File, io::Write};

use log::{error, LevelFilter};
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
        help = "The input files and there translations.\nBy default it takes a list of json strings that represents how the models should be loaded and translated.\nSee simple_input for an alternative command. "
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

    // Output the GCode
    if let Some(file_path) = &args.output {
        let mut profiling_callbacks: Box<dyn PipelineCallbacks> =
            Box::new(ProfilingCallbacks::new());
        // Output to file
        let mut file = handle_err_or_return(
            File::create(file_path).map_err(|_| SlicerErrors::FileCreateError {
                filepath: file_path.to_string(),
            }),
            args.message,
        );
        let (models, settings) = handle_err_or_return(handle_io(&args), args.message);

        handle_err_or_return(
            slicer_pipeline(&models, &settings, &mut profiling_callbacks, &mut file),
            args.message,
        );
    } else if args.message {
        // Output as message
        let mut gcode: Vec<u8> = Vec::new();
        let mut messaging_callbacks: Box<dyn PipelineCallbacks> = Box::new(MessageCallbacks {});

        let (models, settings) = handle_err_or_return(handle_io(&args), args.message);

        handle_err_or_return(
            slicer_pipeline(&models, &settings, &mut messaging_callbacks, &mut gcode),
            args.message,
        );
        let message = Message::GCode(
            String::from_utf8(gcode).expect("All write occur from write macro so should be utf8"),
        );
        bincode::serialize_into(BufWriter::new(std::io::stdout()), &message)
            .expect("Write Limit should not be hit");
    } else {
        // Output to stdout
        let stdout = std::io::stdout();
        let mut stdio_lock = stdout.lock();
        let mut profiling_callbacks: Box<dyn PipelineCallbacks> =
            Box::new(ProfilingCallbacks::new());

        let (models, settings) = handle_err_or_return(handle_io(&args), args.message);

        handle_err_or_return(
            slicer_pipeline(
                &models,
                &settings,
                &mut profiling_callbacks,
                &mut stdio_lock,
            ),
            args.message,
        );
    };
}

fn handle_io(args: &Args) -> Result<(Vec<ModelRawData>, Settings), SlicerErrors> {
    let settings_json = args
        .settings_json
        .as_ref()
        .map(|s| Ok(s.clone()))
        .unwrap_or_else(|| {
            input::load_settings_json(
                args.settings_file_path
                    .as_deref()
                    .expect("CLAP should handle requiring a settings option to be Some"),
            )
        })?;

    let settings = load_settings(args.settings_file_path.as_deref(), &settings_json)?;

    let input_objs: Result<Vec<InputObject>, SlicerErrors> = args
        .input
        .iter()
        .map(|value| {
            if args.simple_input {
                Ok(InputObject::Auto(value.clone()))
            } else {
                deser_hjson::from_str(&value).map_err(|_| SlicerErrors::InputMisformat)
            }
        })
        .collect();

    let models = crate::input::load_models(input_objs?, &settings)?;
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

    Ok((models, settings))
}

fn handle_err_or_return<T>(res: Result<T, SlicerErrors>, message: bool) -> T {
    match res {
        Ok(data) => data,
        Err(slicer_error) => {
            if message {
                let stdout = std::io::stdout();
                let mut stdio_lock = stdout.lock();

                let message = Message::Error(slicer_error);
                bincode::serialize_into(&mut stdio_lock, &message)
                    .expect("Write Limit should not be hit");
                stdio_lock.flush().expect("Standard Out should be limited");
            } else {
                let (error_code, message) = slicer_error.get_code_and_message();
                error!("\n");
                error!("**************************************************");
                error!("\tGladius Slicer Ran into an error");
                error!("\tError Code: {:#X}", error_code);
                error!("\t{}", message);
                error!("**************************************************");
                error!("\n\n\n");
            }
            std::process::exit(-1);
        }
    }
}

pub struct MessageCallbacks {}

impl PipelineCallbacks for MessageCallbacks {
    fn handle_state_update(&mut self, state_message: &str) {
        let stdout = std::io::stdout();
        let mut stdio_lock = stdout.lock();
        let message = Message::StateUpdate(state_message.to_string());
        bincode::serialize_into(&mut stdio_lock, &message).expect("Write Limit should not be hit");
        stdio_lock.flush().expect("Standard Out should be limited");
    }

    fn handle_commands(&mut self, moves: &[Command]) {
        let message = Message::Commands(Vec::from(moves));
        bincode::serialize_into(BufWriter::new(std::io::stdout()), &message)
            .expect("Write Limit should not be hit");
    }

    fn handle_settings_warning(&mut self, warning: SlicerWarnings) {
        let stdout = std::io::stdout();
        let mut stdio_lock = stdout.lock();
        let message = Message::Warning(warning);
        bincode::serialize_into(&mut stdio_lock, &message).expect("Write Limit should not be hit");
        stdio_lock.flush().expect("Standard Out should be limited");
    }

    fn handle_calculated_values(&mut self, cv: CalculatedValues, _settings: &Settings) {
        let message = Message::CalculatedValues(cv);
        bincode::serialize_into(BufWriter::new(std::io::stdout()), &message)
            .expect("Write Limit should not be hit");
    }

    fn handle_slice_finished(&mut self) {}
}
