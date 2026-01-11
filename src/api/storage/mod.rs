mod cold;
mod file;
mod hot;
mod temp;

use crate::config::DATA_DIR as BASE_DATA_DIR;

pub use cold::ColdTable;
pub use file::File;
pub use file::FileBackend;
pub use file::FileStorage;
pub use hot::HotTable;
pub use temp::TempFile;

const BINCODE_CONFIG: bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Fixint,
> = bincode::config::standard().with_fixed_int_encoding();
