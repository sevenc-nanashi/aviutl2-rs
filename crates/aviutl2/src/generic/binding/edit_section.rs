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
    /// 生ポインタから `EditSection` を作成します。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_ptr(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self {
            internal: ptr,
            info: unsafe { EditInfo::from_ptr((*ptr).info) },
        }
    }

    /// オブジェクトエイリアスから指定の位置にオブジェクトを作成します。
    ///
    /// # Arguments
    ///
    /// - `alias`：オブジェクトエイリアスのデータ。オブジェクトエイリアスと同じフォーマットで指定  
    /// - `layer`：作成するオブジェクトのレイヤー番号（0始まり）。
    /// - `frame`：作成するオブジェクトのフレーム番号（0始まり）。
    /// - `length`：作成するオブジェクトの長さ（フレーム数）。
    ///
    /// # Errors
    ///
    /// エイリアスの変換に失敗した場合、またはオブジェクトが既存のオブジェクトに重なる場合にエラー
    ///
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
        Ok(ObjectHandle {
            internal: object_handle,
        })
    }

    /// 指定のフレーム番号以降にあるオブジェクトを検索します。
    ///
    /// # Arguments
    ///
    /// - `layer`：検索するレイヤー番号（0始まり）。
    /// - `frame`：検索を開始するフレーム番号（0始まり）。
    pub fn find_object_after(&self, layer: usize, frame: usize) -> AnyResult<Option<ObjectHandle>> {
        let object_handle =
            unsafe { ((*self.internal).find_object)(layer.try_into()?, frame.try_into()?) };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle {
                internal: object_handle,
            }))
        }
    }

    /// 指定のオブジェクトのレイヤーとフレーム情報を取得します。
    pub fn get_object_layer_frame(&self, object: &ObjectHandle) -> AnyResult<ObjectLayerFrame> {
        self.ensure_object_exists(object)?;
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        Ok(ObjectLayerFrame {
            layer: object.layer.try_into()?,
            start: object.start.try_into()?,
            end: object.end.try_into()?,
        })
    }

    /// オブジェクトの情報をエイリアスデータとして取得します。
    pub fn get_object_alias(&self, object: &ObjectHandle) -> AnyResult<String> {
        self.ensure_object_exists(object)?;
        let alias_ptr = unsafe { ((*self.internal).get_object_alias)(object.internal) };
        if alias_ptr.is_null() {
            anyhow::bail!("Failed to get object alias");
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(alias_ptr) };
        let alias = c_str.to_str()?.to_owned();
        Ok(alias)
    }

    /// オブジェクトの設定項目の値を文字列で取得します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
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
        let value = c_str.to_str()?.to_owned();
        Ok(value)
    }

    /// オブジェクトの設定項目の値を文字列で設定します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    /// - `value`：設定する値。
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
        let success = unsafe {
            ((*self.internal).set_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                c_value.as_ptr(),
            )
        };
        if !success {
            anyhow::bail!("Failed to set object effect item");
        }
        Ok(())
    }

    /// オブジェクトを移動します。
    pub fn move_object(
        &self,
        object: &ObjectHandle,
        new_layer: usize,
        new_start_frame: usize,
    ) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        let success = unsafe {
            ((*self.internal).move_object)(
                object.internal,
                new_layer.try_into()?,
                new_start_frame.try_into()?,
            )
        };
        if !success {
            anyhow::bail!("Failed to move object");
        }
        Ok(())
    }

    /// オブジェクトを削除します。
    pub fn delete_object(&self, object: &ObjectHandle) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).delete_object)(object.internal) };
        Ok(())
    }

    /// 現在、オブジェクト設定ウィンドウで選択されているオブジェクトを取得します。
    pub fn get_focused_object(&self) -> AnyResult<Option<ObjectHandle>> {
        let object_handle = unsafe { ((*self.internal).get_focus_object)() };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle {
                internal: object_handle,
            }))
        }
    }

    /// オブジェクト設定ウィンドウで指定のオブジェクトを選択状態にします。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self, object: &ObjectHandle) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).set_focus_object)(object.internal) };
        Ok(())
    }

    /// ログにメッセージを出力します。
    pub fn output_log(&self, message: &str) -> AnyResult<()> {
        let wide = crate::common::CWString::new(message)?;
        unsafe { ((*self.internal).output_log)(wide.as_ptr()) };
        Ok(())
    }

    /// オブジェクトが存在するかどうか調べます。
    ///
    /// # Note
    ///
    /// 内部的には、get_object_layer_frame を呼び出してレイヤー番号が -1 でないかを確認しています。
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

    /// すべてのレイヤーをイテレータで取得します。
    ///
    /// UI 表示と異なり、レイヤー番号は 0 始まりです。
    pub fn layers(&self) -> EditSectionLayersIterator<'_> {
        EditSectionLayersIterator::new(self)
    }

    /// [EditSectionLayerCaller] を作成します。
    pub fn layer<'a>(&'a self, layer: usize) -> EditSectionLayerCaller<'a> {
        EditSectionLayerCaller::new(self, layer)
    }
    /// [EditSectionObjectCaller] を作成します。
    pub fn object<'a>(&'a self, object: &'a ObjectHandle) -> EditSectionObjectCaller<'a> {
        EditSectionObjectCaller::new(self, object)
    }
}

/// オブジェクト主体で関数を呼び出すための構造体。
/// EditSection と ObjectHandle の組をまとめ、対象オブジェクトに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionObjectCaller<'a> {
    edit_section: &'a EditSection,
    object: &'a ObjectHandle,
}
impl<'a> EditSectionObjectCaller<'a> {
    pub fn new(edit_section: &'a EditSection, object: &'a ObjectHandle) -> Self {
        Self {
            edit_section,
            object,
        }
    }
    /// オブジェクトのレイヤーとフレーム情報を取得します。
    pub fn get_layer_frame(&self) -> AnyResult<ObjectLayerFrame> {
        self.edit_section.get_object_layer_frame(self.object)
    }
    /// オブジェクトの情報をエイリアスデータとして取得します。
    pub fn get_alias(&self) -> AnyResult<String> {
        self.edit_section.get_object_alias(self.object)
    }
    /// オブジェクトの設定項目の値を文字列で取得します。
    ///
    /// # Arguments
    ///
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    pub fn get_effect_item(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> AnyResult<String> {
        self.edit_section
            .get_object_effect_item(self.object, effect_name, effect_index, item)
    }

    /// オブジェクトの設定項目の値を文字列で設定します。
    ///
    /// # Arguments
    ///
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    /// - `value`：設定する値。
    pub fn set_effect_item(
        &self,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        value: &str,
    ) -> AnyResult<()> {
        self.edit_section.set_object_effect_item(
            self.object,
            effect_name,
            effect_index,
            item,
            value,
        )
    }

    /// オブジェクトを移動します。
    ///
    /// # Arguments
    ///
    /// - `new_layer`：移動先のレイヤー番号（0始まり）。
    /// - `new_start_frame`：移動先の開始フレーム番号（0始まり）。
    pub fn move_object(&self, new_layer: usize, new_start_frame: usize) -> AnyResult<()> {
        self.edit_section
            .move_object(self.object, new_layer, new_start_frame)
    }

    /// オブジェクトを削除します。
    pub fn delete_object(&self) -> AnyResult<()> {
        self.edit_section.delete_object(self.object)
    }

    /// オブジェクト設定ウィンドウでこのオブジェクトを選択状態にします。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self) -> AnyResult<()> {
        self.edit_section.focus_object(self.object)
    }

    /// このオブジェクトが存在するかどうか調べます。
    pub fn exists(&self) -> bool {
        self.edit_section.object_exists(self.object)
    }
}

/// レイヤー主体で関数を呼び出すための構造体。
/// EditSection と レイヤー番号 の組をまとめ、対象レイヤーに対する
/// 操作を簡潔に呼び出せるようにします。
pub struct EditSectionLayerCaller<'a> {
    edit_section: &'a EditSection,
    layer: usize,
}
impl<'a> EditSectionLayerCaller<'a> {
    pub fn new(edit_section: &'a EditSection, layer: usize) -> Self {
        Self {
            edit_section,
            layer,
        }
    }
    /// 指定のフレーム番号以降にあるオブジェクトを検索します。
    ///
    /// # Arguments
    ///
    /// - `frame`：検索を開始するフレーム番号（0始まり）。
    pub fn find_object_after(&self, frame: usize) -> AnyResult<Option<ObjectHandle>> {
        self.edit_section.find_object_after(self.layer, frame)
    }

    /// このレイヤーに存在するすべてのオブジェクトを、
    /// 開始フレームの昇順で走査するイテレータを返します。
    pub fn objects(&self) -> EditSectionLayerObjectsIterator<'a> {
        EditSectionLayerObjectsIterator::new(self.edit_section, self.layer)
    }
}

/// レイヤーのイテレータ。
#[derive(Debug, Clone)]
pub struct EditSectionLayersIterator<'a> {
    edit_section: &'a EditSection,
    current: usize,
    total: usize,
}

impl<'a> EditSectionLayersIterator<'a> {
    fn new(edit_section: &'a EditSection) -> Self {
        Self {
            edit_section,
            current: 0,
            total: edit_section.info.layer_max,
        }
    }
}

impl<'a> Iterator for EditSectionLayersIterator<'a> {
    type Item = EditSectionLayerCaller<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.total {
            return None;
        }
        let layer = self.current;
        self.current += 1;
        Some(EditSectionLayerCaller::new(self.edit_section, layer))
    }
}

/// レイヤー内のオブジェクトを走査するイテレータ。
/// アイテムは `(オブジェクトのレイヤー・フレーム情報, ハンドル)` の組です。
#[derive(Debug, Clone)]
pub struct EditSectionLayerObjectsIterator<'a> {
    edit_section: &'a EditSection,
    layer: usize,
    next_frame: usize,
}

impl<'a> EditSectionLayerObjectsIterator<'a> {
    fn new(edit_section: &'a EditSection, layer: usize) -> Self {
        Self {
            edit_section,
            layer,
            next_frame: 0,
        }
    }
}

impl<'a> Iterator for EditSectionLayerObjectsIterator<'a> {
    type Item = (ObjectLayerFrame, ObjectHandle);

    fn next(&mut self) -> Option<Self::Item> {
        // 検索・取得でエラーが出た場合は None を返して終了する。
        let handle = match self
            .edit_section
            .find_object_after(self.layer, self.next_frame)
        {
            Ok(Some(h)) => h,
            _ => return None,
        };

        let lf = match self.edit_section.get_object_layer_frame(&handle) {
            Ok(lf) => lf,
            Err(_) => return None,
        };

        // 次の検索開始位置を、いま見つかったオブジェクトの末尾+1 に進める。
        self.next_frame = lf.end.saturating_add(1);

        Some((lf, handle))
    }
}

trait MenuCallbackReturn {
    fn into_optional_error(self) -> Option<String>;
}
impl<E> MenuCallbackReturn for Result<(), E>
where
    Box<dyn std::error::Error>: From<E>,
{
    fn into_optional_error(self) -> Option<String> {
        match self {
            Ok(_) => None,
            Err(e) => {
                let boxed: Box<dyn std::error::Error> = e.into();
                Some(format!("{}", boxed))
            }
        }
    }
}
impl MenuCallbackReturn for () {
    fn into_optional_error(self) -> Option<String> {
        None
    }
}
