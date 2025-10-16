#[test]
fn test_basic() {
    #[derive(Debug, Clone)]
    struct TestFilter;

    impl aviutl2::filter::FilterPlugin for TestFilter {
        fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
            Ok(Self)
        }

        fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
            aviutl2::filter::FilterPluginTable {
                name: "Test Filter".to_string(),
                label: None,
                information: "This is a test filter plugin for AviUtl2".to_string(),
                filter_type: aviutl2::filter::FilterType::Video,
                as_object: true,
                config_items: vec![],
            }
        }
    }

    let filter = aviutl2_dummy::filter::DummyFilterHost::<TestFilter>::new(
        aviutl2::AviUtl2Version::new(2, 0, 15, 0),
    )
    .unwrap();
    let table = filter.plugin_info();
    assert!(table.name.contains("Test Filter"));
    assert_eq!(table.filter_type, aviutl2::filter::FilterType::Video);
    assert!(table.information.contains("This is a test filter plugin"));
}

#[test]
fn test_configs() {
    use aviutl2::filter::FilterConfigItemSliceExt;
    #[derive(Debug, Clone, aviutl2::filter::FilterConfigItems)]
    #[allow(dead_code)]
    struct TestConfig {
        #[track(name = "数値", range = 0.0..=1.0, step = 0.01, default = 0.5)]
        value: f64,
        #[check(name = "チェック", default = true)]
        check: bool,
        #[select(
            name = "選択肢",
            items = TestEnum,
            default = TestEnum::OptionA
        )]
        choice: TestEnum,
        #[color(name = "色", default = "#ff0000")]
        color: aviutl2::filter::FilterConfigColorValue,
        #[file(name = "ファイル", filters = { "All Files" => [] })]
        file: Option<std::path::PathBuf>,
    }

    #[derive(Debug, Clone, PartialEq, aviutl2::filter::FilterConfigSelectItems)]
    enum TestEnum {
        #[item(name = "オプションA")]
        OptionA,
        #[item(name = "オプションB")]
        OptionB,
        #[item(name = "オプションC")]
        OptionC,
    }

    #[derive(Debug, Clone)]
    struct TestFilter;

    impl aviutl2::filter::FilterPlugin for TestFilter {
        fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
            Ok(Self)
        }

        fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
            use aviutl2::filter::FilterConfigItems;

            aviutl2::filter::FilterPluginTable {
                name: "Test Config Filter".to_string(),
                label: None,
                information: "This is a test filter plugin with configs".to_string(),
                filter_type: aviutl2::filter::FilterType::Video,
                as_object: true,
                config_items: TestConfig::to_config_items(),
            }
        }
    }

    let filter = aviutl2_dummy::filter::DummyFilterHost::<TestFilter>::new(
        aviutl2::AviUtl2Version::new(2, 0, 15, 0),
    )
    .unwrap();
    let table = filter.plugin_info();
    assert!(table.config_items.len() == 5);
    let config = table.config_items.as_slice().to_struct::<TestConfig>();
    assert_eq!(config.value, 0.5);
    assert!(config.check);
    assert_eq!(config.choice, TestEnum::OptionA);
    assert_eq!(config.color.to_rgb(), (255, 0, 0));
    assert_eq!(config.file, None);
}

#[test]
fn test_video_filter() {
    #[derive(Debug, Clone)]
    struct TestFilter;

    impl aviutl2::filter::FilterPlugin for TestFilter {
        fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
            Ok(Self)
        }

        fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
            aviutl2::filter::FilterPluginTable {
                name: "Test Video Filter".to_string(),
                label: None,
                information: "This is a test video filter plugin".to_string(),
                filter_type: aviutl2::filter::FilterType::Video,
                as_object: true,
                config_items: vec![],
            }
        }

        fn proc_video(
            &self,
            _config: &[aviutl2::filter::FilterConfigItem],
            video: &mut aviutl2::filter::FilterProcVideo,
        ) -> aviutl2::AnyResult<()> {
            let mut video_data = vec![
                aviutl2::filter::RgbaPixel::default();
                video.video_object.width as usize
                    * video.video_object.height as usize
            ];
            video.get_image_data(&mut video_data);
            for pixel in &mut video_data {
                // Invert colors
                pixel.r = 255 - pixel.r;
                pixel.g = 255 - pixel.g;
                pixel.b = 255 - pixel.b;
            }
            video.set_image_data(
                &video_data,
                video.video_object.width,
                video.video_object.height,
            );
            Ok(())
        }
    }

    let filter = aviutl2_dummy::filter::DummyFilterHost::<TestFilter>::new(
        aviutl2::AviUtl2Version::new(2, 0, 15, 0),
    )
    .unwrap();
    let scene = aviutl2::filter::SceneInfo {
        width: 2,
        height: 2,
        frame_rate: aviutl2::Rational32::new(30, 1),
        sample_rate: 44100,
    };
    let object = aviutl2::filter::ObjectInfo {
        id: 1,
        frame: 0,
        frame_total: 1,
        time: 0.0,
        time_total: 0.01,
    };
    let video_object = aviutl2::filter::VideoObjectInfo {
        width: 2,
        height: 2,
    };
    let frame_data: Vec<u8> = vec![
        255, 0, 0, 255, // Red
        0, 255, 0, 255, // Green
        0, 0, 255, 255, // Blue
        255, 255, 0, 255, // Yellow
    ];
    let (width, height, data) = filter
        .proc_video(&scene, &object, &video_object, &frame_data)
        .unwrap();
    assert_eq!(width, 2);
    assert_eq!(height, 2);
    assert_eq!(
        data,
        vec![
            0, 255, 255, 255, // Cyan
            255, 0, 255, 255, // Magenta
            255, 255, 0, 255, // Yellow
            0, 0, 255, 255, // Blue
        ]
    );
}

#[test]
fn test_audio_filter() {
    #[derive(Debug, Clone)]
    struct TestFilter;

    impl aviutl2::filter::FilterPlugin for TestFilter {
        fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
            Ok(Self)
        }

        fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
            aviutl2::filter::FilterPluginTable {
                name: "Test Audio Filter".to_string(),
                label: None,
                information: "This is a test audio filter plugin".to_string(),
                filter_type: aviutl2::filter::FilterType::Audio,
                as_object: true,
                config_items: vec![],
            }
        }

        fn proc_audio(
            &self,
            _config: &[aviutl2::filter::FilterConfigItem],
            audio: &mut aviutl2::filter::FilterProcAudio,
        ) -> aviutl2::AnyResult<()> {
            let sample_num = audio.audio_object.sample_num as usize;
            let mut left = vec![0.0; sample_num];
            let mut right = vec![0.0; sample_num];
            audio.get_sample_data(aviutl2::filter::AudioChannel::Left, &mut left);
            audio.get_sample_data(aviutl2::filter::AudioChannel::Right, &mut right);
            for i in 0..sample_num {
                // Simple volume reduction
                left[i] *= 0.5;
                right[i] *= 0.5;
            }
            audio.set_sample_data(aviutl2::filter::AudioChannel::Left, &left);
            audio.set_sample_data(aviutl2::filter::AudioChannel::Right, &right);
            Ok(())
        }
    }

    let filter = aviutl2_dummy::filter::DummyFilterHost::<TestFilter>::new(
        aviutl2::AviUtl2Version::new(2, 0, 15, 0),
    )
    .unwrap();
    let scene = aviutl2::filter::SceneInfo {
        width: 0,
        height: 0,
        frame_rate: aviutl2::Rational32::new(30, 1),
        sample_rate: 44100,
    };
    let object = aviutl2::filter::ObjectInfo {
        id: 1,
        frame: 0,
        frame_total: 1,
        time: 0.0,
        time_total: 0.01,
    };
    let audio_object = aviutl2::filter::AudioObjectInfo {
        sample_num: 4,
        sample_index: 0,
        sample_total: 4,
        channel_num: 2,
    };
    let left_frame_data: Vec<f32> = vec![
        1.0, 0.5, 0.25, 0.125, // Left channel
    ];
    let right_frame_data: Vec<f32> = vec![
        0.0, 0.5, 0.75, 1.0, // Right channel
    ];

    let (left_data, right_data) = filter
        .proc_audio(
            &scene,
            &object,
            &audio_object,
            &left_frame_data,
            &right_frame_data,
        )
        .unwrap();

    assert_eq!(left_data, vec![0.5, 0.25, 0.125, 0.0625]);
    assert_eq!(right_data, vec![0.0, 0.25, 0.375, 0.5]);
}
