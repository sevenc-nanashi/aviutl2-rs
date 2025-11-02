pub mod singleton_traits {
    #[cfg(feature = "input")]
    pub use crate::input::__bridge::InputSingleton as InputPlugin;

    #[cfg(feature = "output")]
    pub use crate::output::__bridge::OutputSingleton as OutputPlugin;

    #[cfg(feature = "filter")]
    pub use crate::filter::__bridge::FilterSingleton as FilterPlugin;

    #[cfg(feature = "module")]
    pub use crate::module::__bridge::ScriptModuleSingleton as ScriptModule;

    #[cfg(feature = "generic")]
    pub use crate::generic::__bridge::GenericSingleton as GenericPlugin;
}

pub mod state {
    #[cfg(feature = "input")]
    pub use crate::input::__bridge::InternalInputPluginState as InputPlugin;

    #[cfg(feature = "output")]
    pub use crate::output::__bridge::InternalOutputPluginState as OutputPlugin;

    #[cfg(feature = "filter")]
    pub use crate::filter::__bridge::InternalFilterPluginState as FilterPlugin;

    #[cfg(feature = "module")]
    pub use crate::module::__bridge::InternalScriptModuleState as ScriptModule;

    #[cfg(feature = "generic")]
    pub use crate::generic::__bridge::InternalGenericPluginState as GenericPlugin;
}
