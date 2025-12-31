mod cold;
mod hot;
mod temp;

use crate::config::DATA_DIR as BASE_DATA_DIR;
use std::fs;

pub use hot::HotTable;
use once_cell::sync::Lazy;
