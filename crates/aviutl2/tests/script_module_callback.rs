fn assert_callback(_: aviutl2::module::ScriptModuleFunctionCallback) {}
fn assert_meta_table(_: aviutl2::module::ScriptModuleMetaTable) {}

#[test]
fn script_module_callback_typechecks() {
    let mut offset = 1;
    let callback = aviutl2::module::script_module_callback!(move |value: i32| -> i32 {
        offset += value;
        offset
    });
    assert_callback(callback);
}

#[test]
fn script_module_direct_callback_typechecks() {
    let callback = aviutl2::module::script_module_direct_callback!(
        move |handle: &mut aviutl2::module::ScriptModuleCallHandle| {
            let value: i32 = handle.get_param(0).unwrap_or(0);
            let _ = handle.push_result(value + 1);
        }
    );
    assert_callback(callback);
}

#[test]
fn script_module_meta_table_typechecks() {
    let meta_table = aviutl2::module::ScriptModuleMetaTable::new(
        aviutl2::module::script_module_direct_callback!(
            move |handle: &mut aviutl2::module::ScriptModuleCallHandle| {
                let key = handle.get_param_str(1).unwrap_or_default();
                let _ = handle.push_result(key.len() as i32);
            }
        ),
        aviutl2::module::script_module_direct_callback!(
            move |handle: &mut aviutl2::module::ScriptModuleCallHandle| {
                let key = handle.get_param_str(1).unwrap_or_default();
                let value = handle.get_param_str(2).unwrap_or_default();
                let _ = (key, value);
            }
        ),
    );
    assert_meta_table(meta_table);
}

#[test]
fn script_module_meta_table_return_value_typechecks() {
    fn make_meta_table() -> impl aviutl2::module::IntoScriptModuleReturnValue {
        aviutl2::module::ScriptModuleMetaTable::new(
            aviutl2::module::script_module_direct_callback!(
                move |handle: &mut aviutl2::module::ScriptModuleCallHandle| {
                    let key = handle.get_param_str(1).unwrap_or_default();
                    let _ = handle.push_result(key.len() as i32);
                }
            ),
            aviutl2::module::script_module_direct_callback!(
                move |_handle: &mut aviutl2::module::ScriptModuleCallHandle| {}
            ),
        )
    }

    let _ = make_meta_table();
}
