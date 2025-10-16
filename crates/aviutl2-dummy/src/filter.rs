use zerocopy::IntoBytes;

/// ダミーのフィルタプラグインのホスト。
pub struct DummyFilterHost<T: aviutl2::filter::FilterPlugin> {
    state: aviutl2::filter::__bridge::InternalFilterPluginState<T>,
    items_ptr: *const aviutl2::sys::filter2::FILTER_ITEM,
    table: aviutl2::filter::FilterPluginTable,
    filter_type: aviutl2::filter::FilterType,
}

impl<T: aviutl2::filter::FilterPlugin> DummyFilterHost<T> {
    /// 新しいホストを作成します。
    ///
    /// # Errors
    ///
    /// `T::new` がエラーを返した場合、そのエラーを返します。
    pub fn new(version: aviutl2::AviUtl2Version) -> aviutl2::AnyResult<Self> {
        let mut state = aviutl2::filter::__bridge::InternalFilterPluginState::new(T::new(
            aviutl2::AviUtl2Info { version },
        )?);

        let plugin_table = unsafe {
            aviutl2::filter::__bridge::create_table(&mut state, func_proc_video, func_proc_audio)
        };
        let items_ptr =
            unsafe { *(plugin_table.items as *const *const aviutl2::sys::filter2::FILTER_ITEM) };
        let table = aviutl2::filter::FilterPluginTable::from_raw(&plugin_table);
        let filter_type = table.filter_type;
        Ok(Self {
            state,
            items_ptr,
            table,
            filter_type,
        })
    }

    /// フィルタの情報を取得します。
    pub fn plugin_info(&self) -> &aviutl2::filter::FilterPluginTable {
        &self.table
    }

    /// フィルタの設定を変更します。
    pub fn set_config(&mut self, config: &[aviutl2::filter::FilterConfigItem]) {
        let _ = config;
        let _ = self.items_ptr;
        todo!()
    }

    /// 音声フィルタ処理を行います。
    ///
    /// # Panics
    #[allow(clippy::result_unit_err)]
    pub fn proc_audio(
        &self,
        scene_info: &aviutl2::filter::SceneInfo,
        object_info: &aviutl2::filter::ObjectInfo,
        audio_info: &aviutl2::filter::AudioObjectInfo,
        audio: &[f32],
    ) -> Result<(Vec<f32>, Vec<f32>), ()> {
        if self.filter_type != aviutl2::filter::FilterType::Audio
            && self.filter_type != aviutl2::filter::FilterType::Both
        {
            panic!("This filter does not support audio processing");
        }
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

    #[allow(clippy::result_unit_err)]
    pub fn proc_video<I: zerocopy::IntoBytes + zerocopy::FromBytes + zerocopy::Immutable + Copy>(
        &self,
        scene_info: &aviutl2::filter::SceneInfo,
        object_info: &aviutl2::filter::ObjectInfo,
        video_object_info: &aviutl2::filter::VideoObjectInfo,
        image: &[I],
    ) -> Result<(i32, i32, Vec<I>), ()> {
        if self.filter_type != aviutl2::filter::FilterType::Video
            && self.filter_type != aviutl2::filter::FilterType::Both
        {
            panic!("This filter does not support audio processing");
        }
        assert_eq!(
            std::mem::size_of_val(image),
            (video_object_info.width
                * video_object_info.height
                * std::mem::size_of::<aviutl2_sys::filter2::PIXEL_RGBA>() as u32)
                as usize,
            "Image buffer size does not match the specified width and height"
        );
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap();

        static IMAGE_BUFFER: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(vec![]);
        static RETURN_BUFFER: std::sync::Mutex<(i32, i32, Vec<u8>)> =
            std::sync::Mutex::new((0, 0, vec![]));

        {
            let mut image_buffer = IMAGE_BUFFER.lock().unwrap();
            image_buffer.clear();
            let bytes = image.as_bytes();
            image_buffer.extend_from_slice(bytes);
        }
        {
            let mut return_buffer = RETURN_BUFFER.lock().unwrap();
            return_buffer.0 = 0;
            return_buffer.1 = 0;
            return_buffer.2.clear();
        }

        let mut video = aviutl2_sys::filter2::FILTER_PROC_VIDEO {
            scene: &scene_info.to_raw(),
            object: &object_info.to_raw(Some(video_object_info), None),
            get_image_data,
            set_image_data,
        };
        return if unsafe { aviutl2::filter::__bridge::func_proc_video(&self.state, &mut video) } {
            let return_buffer = &*RETURN_BUFFER.lock().unwrap();
            let returning = <[I] as zerocopy::FromBytes>::ref_from_bytes(&return_buffer.2)
                .expect("Failed to convert image buffer");
            Ok((return_buffer.0, return_buffer.1, returning.to_vec()))
        } else {
            Err(())
        };

        extern "C" fn get_image_data(buffer: *mut aviutl2_sys::filter2::PIXEL_RGBA) {
            let image_buffer = &*IMAGE_BUFFER.lock().unwrap();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    image_buffer.as_ptr(),
                    buffer as *mut u8,
                    image_buffer.len(),
                );
            }
        }
        extern "C" fn set_image_data(
            buffer: *const aviutl2_sys::filter2::PIXEL_RGBA,
            width: i32,
            height: i32,
        ) {
            let mut return_buffer = RETURN_BUFFER.lock().unwrap();
            let size =
                (width * height * std::mem::size_of::<aviutl2_sys::filter2::PIXEL_RGBA>() as i32)
                    as usize;
            return_buffer.0 = width;
            return_buffer.1 = height;
            let image_buffer = &mut return_buffer.2;
            image_buffer.clear();
            image_buffer.reserve_exact(size);
            unsafe {
                std::ptr::copy_nonoverlapping(buffer as *const u8, image_buffer.as_mut_ptr(), size);
                image_buffer.set_len(size);
            }
        }
    }
}
extern "C" fn func_proc_audio(_video: *mut aviutl2::sys::filter2::FILTER_PROC_AUDIO) -> bool {
    unreachable!()
}
extern "C" fn func_proc_video(_video: *mut aviutl2::sys::filter2::FILTER_PROC_VIDEO) -> bool {
    unreachable!()
}
