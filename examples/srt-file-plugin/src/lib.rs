use aviutl2::{AnyResult, odbg};

#[aviutl2::plugin(GenericPlugin)]
struct SrtImportPlugin;

impl aviutl2::generic::GenericPlugin for SrtImportPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(SrtImportPlugin)
    }

    fn register(&self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_menus::<SrtImportPlugin>();
        let handle = registry.create_edit_handle();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let res = handle.call_edit_section(|section| {
                odbg!(section);

                format!("Hello from main thread! thread id: {:?}", std::thread::current().id())
            });
            let current_thread_id = std::thread::current().id();
            odbg!(current_thread_id);
            odbg!(res);
        });
    }
}

#[aviutl2::generic::menus]
impl SrtImportPlugin {
    #[import(name = "SRTファイル（*.srt）")]
    fn import_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        todo!()
    }

    #[export(name = "SRTを書き出し（*.srt）")]
    fn export_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        todo!()
    }
}

aviutl2::register_generic_plugin!(SrtImportPlugin);
