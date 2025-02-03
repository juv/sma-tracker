use env_logger::Builder;
use log::LevelFilter;

pub fn init_logger() {
    Builder::new().filter(None, LevelFilter::Debug).init();
}
