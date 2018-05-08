use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fmt;
use std::process;

use boids::{SimulationConfig, WindowSize};

use clap::{self, App, Arg, ArgMatches};
use clap::ErrorKind::{VersionDisplayed, HelpDisplayed};
use serde::de::Deserialize;
use toml;

const CONFIG_ARG: &str = "config";
const WINDOW_SIZE_ARG: &str = "size";
const FULLSCREEN_ARG: &str = "fullscreen";
const BOID_COUNT_ARG: &str = "boids";
const DEBUG_ARG: &str = "debug";

pub fn build_config() -> Result<SimulationConfig, ConfigError> {
    let mut builder = ConfigBuilder::new();

    let cli_args = parse_cli_args()?;

    if let Some(path) = cli_args.value_of(CONFIG_ARG) {
        builder.apply(UserConfig::from_toml_file(path)?);
    }
    builder.apply(UserConfig::from_cli_args(&cli_args)?);

    Ok(builder.build())

}

struct ConfigBuilder {
    config: SimulationConfig,
}

impl ConfigBuilder {

    fn new() -> Self {
        ConfigBuilder{ config: SimulationConfig::default() }
    }

    fn apply(&mut self, uc: UserConfig) {
        let c = &mut self.config;
        merge(&mut c.boid_count,  uc.boid_count);
        merge(&mut c.debug,       uc.debug);
        merge(&mut c.window_size, uc.window_size());

    }

    fn build(self) -> SimulationConfig {
        self.config
    }

}


fn merge<T>(existing: &mut T, candidate: Option<T>)  {
    if let Some(v) = candidate {
        *existing = v;
    }
}

//TODO: Would be cool if there was an arg to print / generate an example config file
fn parse_cli_args() -> Result<ArgMatches<'static>, clap::Error> {
    let args = App::new("boid-simulator")
        .version("0.1")
        .author("James Green")
        .about("Simulates flocking behaviour of birds")
        .arg(Arg::with_name(CONFIG_ARG)
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets the config file to read simulation parameters from"))
        .arg(Arg::with_name(WINDOW_SIZE_ARG)
             .short("s")
             .long("size")
             .value_names(&["width", "height"])
             .use_delimiter(true)
             .help("Sets the simultion window to specified width & height"))
        .arg(Arg::with_name(FULLSCREEN_ARG)
             .short("f")
             .long("fullscreen")
             .help("Display fullscreen (overrides size argument)")
             .conflicts_with("size"))
        .arg(Arg::with_name(BOID_COUNT_ARG)
             .short("b")
             .long("boid-count")
             .takes_value(true)
             .help("Sets the number of boids to simulate"))
        .arg(Arg::with_name(DEBUG_ARG)
             .short("d")
             .long("debug")
             .help("print opengl debug information"))
        .get_matches_safe();

    //TODO: Cleaner as an if let statement?
    match args {
        Err(ref err) if (err.kind == VersionDisplayed) |
                        (err.kind == HelpDisplayed) => err.exit(),
        _ => args
    }
}


#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Clap(clap::Error),
    Toml(toml::de::Error),
}

impl From<io::Error> for ConfigError {
   fn from(err: io::Error) -> ConfigError {
       ConfigError::Io(err)
   }
}

impl From<clap::Error> for ConfigError {
    fn from(err: clap::Error) -> ConfigError {
        ConfigError::Clap(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> ConfigError {
        ConfigError::Toml(err)
    }

}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::Io(ref err) =>
                write!(f, "Could not read config:\n\n{}", err),
            ConfigError::Clap(ref err) =>
                write!(f, "Could not parse arguments:\n\n{}", err),
            ConfigError::Toml(ref err) =>
                write!(f, "Could not parse toml:\n\n{}", err),
        }
    }
}

impl ConfigError {
   pub fn exit(&self) -> ! {
        println!("{}", self);
        process::exit(1);
   }
}



#[derive(Deserialize)]
struct UserConfig {
    boid_count: Option<u32>,
    debug: Option<bool>,
    window: Option<WindowConfig>,
}

#[derive(Copy, Clone, Deserialize)]
struct WindowConfig {
    size: Option<(u32, u32)>,
    fullscreen: Option<bool>,
}


impl UserConfig {

    fn from_toml_file(path: &str) -> Result<Self, ConfigError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }

    fn from_cli_args(args: &ArgMatches<'static>) -> Result<Self, ConfigError> {

        //TODO: Make this method a bit cleaner
        let fullscreen = match args.is_present(FULLSCREEN_ARG) {
            true  => Some(true),
            false => None,
        };

        let size = match args.is_present(WINDOW_SIZE_ARG) {
            true => {
                let size = values_t!(args, WINDOW_SIZE_ARG, u32)?;
                Some((size[0], size[1]))
            }
            false => None,
        };

        let boid_count = match args.is_present(BOID_COUNT_ARG) {
            true  => Some(value_t!(args, BOID_COUNT_ARG, u32)?),
            _     => None,
        };

        let debug = match args.is_present(DEBUG_ARG) {
            true  => Some(true),
            _     => None,
        };

        let window = Some(WindowConfig{fullscreen, size});

        Ok(UserConfig{
            boid_count,
            debug,
            window,
            ..Default::default()
        })
    }

    fn window_size(&self) -> Option<WindowSize> {
        match self.window {
           Some(WindowConfig{ fullscreen:Some(true), ..}) => Some(WindowSize::Fullscreen),
           Some(WindowConfig{ size:Some(dims), ..}) => Some(WindowSize::Dimensions(dims)),
           _ => None,
        }
    }
}

impl Default for UserConfig {
    fn default() -> UserConfig {
        UserConfig {
            boid_count: None,
            window: None,
            debug: None,
        }
    }
}
