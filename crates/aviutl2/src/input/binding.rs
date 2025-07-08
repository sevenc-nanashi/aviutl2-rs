pub struct InputPluginTable {
    pub name: String,
    pub filefilter: String,
    pub information: String,

    pub can_config: bool,
}

pub struct InputInfo {
    pub flag: i32,
    pub rate: i32,
    pub scale: i32,
    pub num_frames: i32,
    pub num_samples: i32,
}

pub trait InputPlugin: Send + Sync {
    type InputHandle: std::any::Any + Send + Sync;

    fn new() -> Self;

    fn info(&self) -> InputPluginTable;

    fn open(&self, file: std::path::PathBuf) -> Option<Self::InputHandle>;
    fn get_info(&self, handle: &Self::InputHandle) -> Result<InputInfo, String>;

    fn close(&self, handle: &mut Self::InputHandle) -> bool;
}
