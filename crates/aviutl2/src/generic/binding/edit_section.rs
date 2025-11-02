use crate::common::{AnyResult, Rational32};

/// オブジェクトへのハンドル。
#[derive(Debug, Clone, Copy)]
pub struct ObjectHandle {
    pub(crate) internal: aviutl2_sys::plugin2::OBJECT_HANDLE,
}
impl From<aviutl2_sys::plugin2::OBJECT_HANDLE> for ObjectHandle {
    fn from(value: aviutl2_sys::plugin2::OBJECT_HANDLE) -> Self {
        Self { internal: value }
    }
}
impl From<ObjectHandle> for aviutl2_sys::plugin2::OBJECT_HANDLE {
    fn from(value: ObjectHandle) -> Self {
        value.internal
    }
}
unsafe impl Send for ObjectHandle {}
unsafe impl Sync for ObjectHandle {}

/// 編集情報構造体。
///
/// # Note
///
/// UI表示と異なり、フレーム番号・レイヤー番号は0始まりです。
#[derive(Debug, Clone, Copy)]
pub struct EditInfo {
    pub width: usize,
    pub height: usize,
    pub fps: Rational32,
    pub sample_rate: usize,
    pub frame: usize,
    pub layer: usize,
    pub frame_max: usize,
    pub layer_max: usize,
}

impl EditInfo {
    /// # Safety
    pub unsafe fn from_ptr(ptr: *const aviutl2_sys::plugin2::EDIT_INFO) -> Self {
        let raw = unsafe { &*ptr };
        Self {
            width: raw.width as usize,
            height: raw.height as usize,
            fps: Rational32::new(raw.rate, raw.scale),
            sample_rate: raw.sample_rate as usize,
            frame: raw.frame as usize,
            layer: raw.layer as usize,
            frame_max: raw.frame_max as usize,
            layer_max: raw.layer_max as usize,
        }
    }
}

/// オブジェクトのレイヤーとフレーム情報。
#[derive(Debug, Clone, Copy)]
pub struct ObjectLayerFrame {
    pub layer: usize,
    pub start: usize,
    pub end: usize,
}

/// 編集セクションのハンドル。
#[derive(Debug)]
pub struct EditSection {
    pub info: EditInfo,
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
}

impl EditSection {
    /// # Safety
    pub unsafe fn from_ptr(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self {
            internal: ptr,
            info: unsafe { EditInfo::from_ptr((*ptr).info) },
        }
    }

    pub fn create_object_from_alias(
        &self,
        alias: &str,
        layer: usize,
        frame: usize,
        length: usize,
    ) -> AnyResult<ObjectHandle> {
        let c_alias = std::ffi::CString::new(alias)?;
        let object_handle = unsafe {
            ((*self.internal).create_object_from_alias)(
                c_alias.as_ptr(),
                layer.try_into()?,
                frame.try_into()?,
                length.try_into()?,
            )
        };
        if object_handle.is_null() {
            anyhow::bail!("Failed to create object from alias");
        }
        Ok(ObjectHandle { internal: object_handle })
    }

    pub fn find_object_after(&self, layer: usize, frame: usize) -> AnyResult<Option<ObjectHandle>> {
        let object_handle =
            unsafe { ((*self.internal).find_object)(layer.try_into()?, frame.try_into()?) };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle { internal: object_handle }))
        }
    }

    pub fn get_object_layer_frame(&self, object: &ObjectHandle) -> AnyResult<ObjectLayerFrame> {
        self.ensure_object_exists(object)?;
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        Ok(ObjectLayerFrame {
            layer: object.layer.try_into()?,
            start: object.start.try_into()?,
            end: object.end.try_into()?,
        })
    }

    pub fn get_object_alias(&self, object: &ObjectHandle) -> AnyResult<String> {
        self.ensure_object_exists(object)?;
        let alias_ptr = unsafe { ((*self.internal).get_object_alias)(object.internal) };
        if alias_ptr.is_null() {
            anyhow::bail!("Failed to get object alias");
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(alias_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn get_object_effect_item(
        &self,
        object: &ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> AnyResult<String> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&format!("{effect_name}:{effect_index}"))?;
        let c_item = crate::common::CWString::new(item)?;
        let value_ptr = unsafe {
            ((*self.internal).get_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
            )
        };
        if value_ptr.is_null() {
            anyhow::bail!("Failed to get object effect item");
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    pub fn set_object_effect_item(
        &self,
        object: &ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        value: &str,
    ) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&format!("{effect_name}:{effect_index}"))?;
        let c_item = crate::common::CWString::new(item)?;
        let c_value = std::ffi::CString::new(value)?;
        let ok = unsafe {
            ((*self.internal).set_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                c_value.as_ptr(),
            )
        };
        if ok { Ok(()) } else { anyhow::bail!("Failed to set object effect item") }
    }

    pub fn move_object(&self, object: &ObjectHandle, layer: usize, frame: usize) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        let ok = unsafe {
            ((*self.internal).move_object)(object.internal, layer.try_into()?, frame.try_into()?)
        };
        if ok { Ok(()) } else { anyhow::bail!("Failed to move object") }
    }

    pub fn delete_object(&self, object: &ObjectHandle) -> AnyResult<()> {
        if self.get_object_layer_frame(object).is_err() {
            anyhow::bail!("Object does not exist in the current edit section");
        }
        unsafe { ((*self.internal).delete_object)(object.internal) };
        Ok(())
    }

    pub fn get_focused_object(&self) -> AnyResult<Option<ObjectHandle>> {
        let object_handle = unsafe { ((*self.internal).get_focus_object)() };
        if object_handle.is_null() { Ok(None) } else { Ok(Some(ObjectHandle { internal: object_handle })) }
    }

    pub fn focus_object(&self, object: &ObjectHandle) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).set_focus_object)(object.internal) };
        Ok(())
    }

    pub fn output_log(&self, message: &str) -> AnyResult<()> {
        let wide = crate::common::CWString::new(message)?;
        unsafe { ((*self.internal).output_log)(wide.as_ptr()) };
        Ok(())
    }

    pub fn object_exists(&self, object: &ObjectHandle) -> bool {
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        object.layer != -1
    }

    fn ensure_object_exists(&self, object: &ObjectHandle) -> AnyResult<()> {
        if !self.object_exists(object) {
            anyhow::bail!("Object does not exist in the current edit section");
        }
        Ok(())
    }

    #[doc(hidden)]
    #[expect(private_bounds)]
    pub fn __output_log_if_error<T: MenuCallbackReturn>(&self, result: T) {
        if let Some(err_msg) = result.into_optional_error() {
            let _ = self.output_log(&err_msg);
        }
    }
}

/// 編集ハンドル。
#[derive(Debug)]
pub struct EditHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
}

unsafe impl Send for EditHandle {}
unsafe impl Sync for EditHandle {}

impl EditHandle {
    pub(crate) unsafe fn new(internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE) -> Self {
        Self { internal }
    }

    pub fn call_edit_section<T, F>(&self, callback: F) -> AnyResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut EditSection) -> T + Send + 'static,
    {
        type TrampolineCallback = dyn FnOnce(&mut EditSection) -> Box<dyn std::any::Any + Send> + Send;
        static NEXT_CALLBACK: std::sync::Mutex<Option<Box<TrampolineCallback>>> = std::sync::Mutex::new(None);
        static CALLBACK_RETURN_VALUE: std::sync::Mutex<Option<Box<dyn std::any::Any + Send>>> = std::sync::Mutex::new(None);
        {
            let mut guard = NEXT_CALLBACK.lock().unwrap();
            *guard = Some(Box::new(move |section: &mut EditSection| {
                let result = callback(section);
                Box::new(result) as Box<dyn std::any::Any + Send>
            }));
        }
        let call_result = unsafe { ((*self.internal).call_edit_section)(trampoline) };
        if call_result {
            let mut return_guard = CALLBACK_RETURN_VALUE.lock().unwrap();
            if let Some(return_value) = return_guard.take() {
                let boxed: Box<T> = return_value.downcast::<T>().expect("Type mismatch in EditSection callback return value");
                return Ok(*boxed);
            } else {
                unreachable!("No return value from EditSection callback")
            }
        } else {
            anyhow::bail!("call_edit_section failed")
        }

        extern "C" fn trampoline(edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION) {
            let mut guard = NEXT_CALLBACK.lock().unwrap();
            if let Some(callback) = guard.take() {
                let mut section = unsafe { EditSection::from_ptr(edit_section) };
                let return_value = callback(&mut section);
                let mut return_guard = CALLBACK_RETURN_VALUE.lock().unwrap();
                *return_guard = Some(return_value);
            }
        }
    }
}

/// オブジェクト主体で関数を呼び出すための構造体。
pub struct EditSectionObjectCaller<'a> { edit_section: &'a EditSection, object: &'a ObjectHandle }
impl<'a> EditSectionObjectCaller<'a> {
    pub fn new(edit_section: &'a EditSection, object: &'a ObjectHandle) -> Self { Self { edit_section, object } }
    pub fn get_layer_frame(&self) -> AnyResult<ObjectLayerFrame> { self.edit_section.get_object_layer_frame(self.object) }
    pub fn get_alias(&self) -> AnyResult<String> { self.edit_section.get_object_alias(self.object) }
    pub fn get_effect_item(&self, effect_name: &str, effect_index: usize, item: &str) -> AnyResult<String> {
        self.edit_section.get_object_effect_item(self.object, effect_name, effect_index, item)
    }
    pub fn set_effect_item(&self, effect_name: &str, effect_index: usize, item: &str, value: &str) -> AnyResult<()> {
        self.edit_section.set_object_effect_item(self.object, effect_name, effect_index, item, value)
    }
}

trait MenuCallbackReturn { fn into_optional_error(self) -> Option<String>; }
impl<E> MenuCallbackReturn for Result<(), E> where Box<dyn std::error::Error>: From<E> {
    fn into_optional_error(self) -> Option<String> { match self { Ok(_) => None, Err(e) => { let boxed: Box<dyn std::error::Error> = e.into(); Some(format!("{}", boxed)) } } }
}
impl MenuCallbackReturn for () { fn into_optional_error(self) -> Option<String> { None } }

