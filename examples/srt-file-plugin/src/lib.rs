use aviutl2::{AnyResult, odbg};

#[aviutl2::plugin(GenericPlugin)]
struct SrtImportPlugin {
    handle: Option<aviutl2::generic::ObjectHandle>,
}

impl aviutl2::generic::GenericPlugin for SrtImportPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(SrtImportPlugin { handle: None })
    }

    fn register(&self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_menus::<SrtImportPlugin>();
        let handle = registry.create_edit_handle();
    }
}

#[aviutl2::generic::menus]
impl SrtImportPlugin {
    #[import(name = "SRTファイル（*.srt）")]
    fn import_menu(&mut self, edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        for layer in 0..=edit_section.info.layer_max {
            let mut current = 0;
            while let Some(obj) = edit_section.find_object_after(layer, current)? {
                edit_section.output_log(&format!("{obj:?}"))?;
                current = edit_section.get_object_layer_frame(&obj)?.end + 1;
            }
        }
        Ok(())
    }

    #[export(name = "SRTを書き出し（*.srt）")]
    fn export_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        todo!()
    }
}

aviutl2::register_generic_plugin!(SrtImportPlugin);
