use aviutl2::AnyResult;

#[aviutl2::plugin(GenericPlugin)]
struct TxtImportPlugin;

impl aviutl2::generic::GenericPlugin for TxtImportPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(TxtImportPlugin)
    }

    fn register(&self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_menus::<TxtImportPlugin>();
    }
}

#[aviutl2::generic::menus]
impl TxtImportPlugin {
    #[import(name = "テキストファイル（*.txt）")]
    fn import_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        todo!()
    }

    #[export(name = "テキストを書き出し（*.txt）")]
    fn export_menu(edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        todo!()
    }
}

aviutl2::register_generic_plugin!(TxtImportPlugin);
