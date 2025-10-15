pub struct DummyFilterHost<T: aviutl2::filter::FilterPlugin> {
    state: aviutl2::filter::__bridge::InternalFilterPluginState<T>,
    items_ptr: *const aviutl2::sys::filter2::FILTER_ITEM,
}

impl<T: aviutl2::filter::FilterPlugin> DummyFilterHost<T> {
    pub fn new(version: aviutl2::AviUtl2Version) -> aviutl2::AnyResult<Self> {
        Ok(Self {
            state: aviutl2::filter::__bridge::InternalFilterPluginState::new(T::new(
                aviutl2::AviUtl2Info { version },
            )?),
            items_ptr: std::ptr::null(),
        })
    }

    pub fn plugin_info(&mut self) -> aviutl2::filter::FilterPluginTable {
        let plugin_table = unsafe {
            aviutl2::filter::__bridge::create_table(
                &mut self.state,
                func_proc_video,
                func_proc_audio,
            )
        };
        self.items_ptr =
            unsafe { *(plugin_table.items as *const *const aviutl2::sys::filter2::FILTER_ITEM) };
        aviutl2::filter::FilterPluginTable::from_raw(&plugin_table)
    }

    pub fn set_config(&mut self, config: &[aviutl2::filter::FilterConfigItem]) {
    }

    pub fn proc_audio(
        &self,
        scene_info: &aviutl2::filter::SceneInfo,
        object_info: &aviutl2::filter::ObjectInfo,
        audio_info: &aviutl2::filter::AudioObjectInfo,
        audio: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>), ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap();

        static AUDIO_BUFFER: std::sync::Mutex<(Vec<f32>, Vec<f32>)> =
            std::sync::Mutex::new((vec![], vec![]));
        static RETURN_BUFFER: std::sync::Mutex<(Vec<f32>, Vec<f32>)> =
            std::sync::Mutex::new((vec![], vec![]));

        {
            let mut audio_buffer = AUDIO_BUFFER.lock().unwrap();
            audio_buffer.0.clear();
            audio_buffer.1.clear();
            audio_buffer.0.extend_from_slice(audio);
            audio_buffer.1.extend_from_slice(audio);
        }
        {
            let mut return_buffer = RETURN_BUFFER.lock().unwrap();
            return_buffer.0.clear();
            return_buffer.1.clear();
            return_buffer.0.resize(audio.len(), 0.0);
            return_buffer.1.resize(audio.len(), 0.0);
        }

        let mut audio = aviutl2_sys::filter2::FILTER_PROC_AUDIO {
            scene: &scene_info.to_raw(),
            object: &object_info.to_raw(None, Some(audio_info)),
            get_sample_data,
            set_sample_data,
        };
        return if unsafe { aviutl2::filter::__bridge::func_proc_audio(&self.state, &mut audio) } {
            let return_buffer = &*RETURN_BUFFER.lock().unwrap();
            Ok(return_buffer.clone())
        } else {
            Err(())
        };

        extern "C" fn get_sample_data(buffer: *mut f32, channel: i32) {
            let channel = aviutl2::filter::AudioChannel::from(channel);
            let audio_buffer = &*AUDIO_BUFFER.lock().unwrap();
            let source = match channel {
                aviutl2::filter::AudioChannel::Left => &audio_buffer.0,
                aviutl2::filter::AudioChannel::Right => &audio_buffer.1,
                aviutl2::filter::AudioChannel::Any(_) => &audio_buffer.0,
            };
            unsafe {
                std::ptr::copy_nonoverlapping(source.as_ptr(), buffer, source.len());
            }
        }
        extern "C" fn set_sample_data(buffer: *const f32, channel: i32) {
            let channel = aviutl2::filter::AudioChannel::from(channel);
            let mut return_buffer = RETURN_BUFFER.lock().unwrap();
            let target = match channel {
                aviutl2::filter::AudioChannel::Left => &mut return_buffer.0,
                aviutl2::filter::AudioChannel::Right => &mut return_buffer.1,
                aviutl2::filter::AudioChannel::Any(_) => &mut return_buffer.0,
            };
            unsafe {
                std::ptr::copy_nonoverlapping(buffer, target.as_mut_ptr(), target.len());
            }
        }
    }

    pub fn proc_video(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        video: &mut aviutl2::filter::FilterProcVideo,
    ) -> aviutl2::AnyResult<()> {
        todo!()
    }
}
extern "C" fn func_proc_audio(video: *mut aviutl2::sys::filter2::FILTER_PROC_AUDIO) -> bool {
    unreachable!()
}
extern "C" fn func_proc_video(video: *mut aviutl2::sys::filter2::FILTER_PROC_VIDEO) -> bool {
    unreachable!()
}
