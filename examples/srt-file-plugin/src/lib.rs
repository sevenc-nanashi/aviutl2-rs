use aviutl2::AnyResult;

#[aviutl2::plugin(GenericPlugin)]
struct SrtImportPlugin {}

impl aviutl2::generic::GenericPlugin for SrtImportPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(SrtImportPlugin {})
    }

    fn register(&self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_menus::<SrtImportPlugin>();
    }
}

#[aviutl2::generic::menus]
impl SrtImportPlugin {
    #[import(name = "SRTファイル（*.srt）")]
    fn import_menu(&mut self, edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        Ok(())
    }

    #[export(name = "SRTを書き出し（*.srt）")]
    fn export_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        for (i, layer) in edit_section.layers().enumerate() {
            edit_section.output_log(&format!("Layer {}:", i))?;
            for (j, obj) in layer.objects() {
                let alias = edit_section.object(&obj).get_alias()?;
                edit_section.output_log(&format!("  Object {:?}: {}", j, alias))?;
            }
        }
        Ok(())
    }
}

aviutl2::register_generic_plugin!(SrtImportPlugin);
