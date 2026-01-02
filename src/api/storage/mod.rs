mod cold;
mod hot;
mod temp;

use crate::config::DATA_DIR as BASE_DATA_DIR;

pub use hot::HotTable;

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();
