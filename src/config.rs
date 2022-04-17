use lazy_static::{initialize, lazy_static};
use simple_config_parser::{Config, ConfigError};

static mut CONFIG: Config = Config { data: Vec::new() };

macro_rules! get_config {
    ($name:expr) => {
        unsafe { &CONFIG }
            .get($name)
            .expect(concat!("Error getting `", $name, "` from Config"))
    };
}

macro_rules! init_lazy {
    ($($exp:expr),+) => {
        $(initialize(&$exp);)*
    };
}

lazy_static! {
    pub static ref THREAD_COUNT: usize = get_config!("thread-count");
    pub static ref DATA_OUT: String = get_config!("data-out");
    pub static ref SPEED_GRAPH_VALUES: usize = get_config!("speed-graph-values");
    pub static ref UI_FPS: usize = get_config!("ui-fps");
}

pub fn load(path: &str) -> Result<(), ConfigError> {
    let cfg = Config::new().file(path)?;

    unsafe {
        CONFIG = cfg;
    }

    // Init the lazy config values
    init_lazy! {
        THREAD_COUNT, DATA_OUT, SPEED_GRAPH_VALUES, UI_FPS
    }

    Ok(())
}
