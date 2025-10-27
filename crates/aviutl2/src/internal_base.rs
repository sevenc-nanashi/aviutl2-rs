pub mod singleton_traits {
    #[cfg(feature = "input")]
    pub use crate::input::__bridge::InternalInputBridge as InputPlugin;

    #[cfg(feature = "output")]
    pub use crate::output::__bridge::InternalOutputBridge as OutputPlugin;

    #[cfg(feature = "filter")]
    pub use crate::filter::__bridge::FilterSingleton as FilterPlugin;
}

pub mod state {
    #[cfg(feature = "input")]
    pub use crate::input::__bridge::InternalInputPluginState as InputPlugin;

    #[cfg(feature = "output")]
    pub use crate::output::__bridge::InternalOutputPluginState as OutputPlugin;

    #[cfg(feature = "filter")]
    pub use crate::filter::__bridge::InternalFilterPluginState as FilterPlugin;
}
