pub struct DummyFilterHost<T: aviutl2::filter::FilterPlugin> {
    state: aviutl2::filter::__bridge::InternalFilterPluginState<T>,
}

impl<T: aviutl2::filter::FilterPlugin> DummyFilterHost<T> {
    pub fn new(version: aviutl2::AviUtl2Version) -> aviutl2::AnyResult<Self> {
        Ok(Self {
            state: aviutl2::filter::__bridge::InternalFilterPluginState::new(T::new(
                aviutl2::AviUtl2Info { version },
            )?),
        })
    }

    pub fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        let info = self.state.plugin.plugin_info();
    }

    pub fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &mut aviutl2::filter::FilterProcAudio,
    ) -> aviutl2::AnyResult<()> {
        aviutl2::filter::__bridge::func_proc_audio(plugin_state, audio)
    }

    pub fn proc_video(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        video: &mut aviutl2::filter::FilterProcVideo,
    ) -> aviutl2::AnyResult<()> {
        self.internal.proc_video(config, video)
    }
}
