/// 関数の引数・返り値を扱うための型とトレイト。
#[derive(Debug)]
pub struct ScriptModuleCallHandle {
    pub(crate) internal: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM,
}
unsafe impl Send for ScriptModuleCallHandle {}

impl ScriptModuleCallHandle {
    /// ポインタから`ScriptModuleParam`を作成する。
    ///
    /// # Safety
    ///
    /// `ptr`は有効な`SCRIPT_MODULE_PARAM`へのポインタである必要があります。
    pub fn from_ptr(ptr: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM) -> ScriptModuleCallHandle {
        ScriptModuleCallHandle { internal: ptr }
    }

    /// 引数の数を返す。
    pub fn len(&self) -> usize {
        unsafe { ((*self.internal).get_param_num)() as usize }
    }

    /// 引数が与えられていないかを返す。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 引数を整数として取得する。
    ///
    /// # Note
    ///
    /// 引数を取得できない場合は0を返します。
    pub fn get_param_int(&self, index: usize) -> i32 {
        unsafe { ((*self.internal).get_param_int)(index as i32) }
    }

    /// 引数を浮動小数点数として取得する。
    ///
    /// # Note
    ///
    /// 引数を取得できない場合は0.0を返します。
    pub fn get_param_float(&self, index: usize) -> f64 {
        unsafe { ((*self.internal).get_param_double)(index as i32) }
    }

    /// 引数を文字列として取得する。
    pub fn get_param_str(&self, index: usize) -> Option<String> {
        unsafe {
            let c_str = ((*self.internal).get_param_string)(index as i32);
            if c_str.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }

    /// 引数をデータポインタとして取得する。
    ///
    /// # Note
    ///
    /// 引数を取得できない場合は`None`を返します。
    pub fn get_param_data<T>(&self, index: usize) -> Option<*mut T> {
        unsafe {
            let data_ptr = ((*self.internal).get_param_data)(index as i32);
            if data_ptr.is_null() {
                None
            } else {
                Some(data_ptr as *mut T)
            }
        }
    }

    /// 引数のテーブルの要素を整数として取得する。
    ///
    /// # Note
    ///
    /// 引数を取得できない場合は0を返します。
    pub fn get_param_table_int(&self, index: usize, key: &str) -> i32 {
        let c_key = std::ffi::CString::new(key).unwrap();
        unsafe { ((*self.internal).get_param_table_int)(index as i32, c_key.as_ptr()) }
    }

    /// 引数のテーブルの要素を浮動小数点数として取得する。
    ///
    /// # Note
    ///
    /// 引数を取得できない場合は0.0を返します。
    pub fn get_param_table_float(&self, index: usize, key: &str) -> f64 {
        let c_key = std::ffi::CString::new(key).unwrap();
        unsafe { ((*self.internal).get_param_table_double)(index as i32, c_key.as_ptr()) }
    }

    /// 引数のテーブルの要素を文字列として取得する。
    pub fn get_param_table_str(&self, index: usize, key: &str) -> Option<String> {
        let c_key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let c_str = ((*self.internal).get_param_table_string)(index as i32, c_key.as_ptr());
            if c_str.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }

    /// 引数の配列の要素の数を取得する。
    pub fn get_param_array_len(&self, index: usize) -> usize {
        unsafe { ((*self.internal).get_param_array_num)(index as i32) as usize }
    }

    /// 引数の配列の要素を数値として取得する。
    pub fn get_param_array_int(&self, index: usize, array_index: usize) -> i32 {
        unsafe { ((*self.internal).get_param_array_int)(index as i32, array_index as i32) }
    }

    /// 引数の配列の要素を浮動小数点数として取得する。
    pub fn get_param_array_float(&self, index: usize, array_index: usize) -> f64 {
        unsafe { ((*self.internal).get_param_array_double)(index as i32, array_index as i32) }
    }

    /// 引数の配列の要素を文字列として取得する。
    pub fn get_param_array_str(&self, index: usize, array_index: usize) -> Option<String> {
        unsafe {
            let c_str = ((*self.internal).get_param_array_string)(index as i32, array_index as i32);
            if c_str.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }

    /// 関数のエラーを設定する。
    pub fn set_error(&self, message: &str) {
        let c_message = std::ffi::CString::new(message).unwrap();
        unsafe {
            ((*self.internal).set_error)(c_message.as_ptr());
        }
    }

    /// 関数の返り値に整数を追加する。
    pub fn push_result_int(&self, value: i32) {
        unsafe {
            ((*self.internal).push_result_int)(value);
        }
    }

    /// 関数の返り値に浮動小数点数を追加する。
    pub fn push_result_float(&self, value: f64) {
        unsafe {
            ((*self.internal).push_result_double)(value);
        }
    }

    /// 関数の返り値に文字列を追加する。
    pub fn push_result_str(&self, value: &str) {
        let c_value = std::ffi::CString::new(value).unwrap();
        unsafe {
            ((*self.internal).push_result_string)(c_value.as_ptr());
        }
    }

    /// 関数の返り値に整数の連想配列を追加する。
    pub fn push_result_table_int<'a, T>(&self, table: T)
    where
        T: std::iter::IntoIterator<Item = (&'a str, i32)>,
    {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in table {
            let c_key = std::ffi::CString::new(key).unwrap();
            keys.push(c_key);
            values.push(value);
        }
        let key_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_table_int)(
                key_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                values.as_mut_ptr(),
                key_ptrs.len() as i32,
            );
        }
    }

    /// 関数の返り値に浮動小数点数の連想配列を追加する。
    pub fn push_result_table_float<'a, T>(&self, table: T)
    where
        T: std::iter::IntoIterator<Item = (&'a str, f64)>,
    {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in table {
            let c_key = std::ffi::CString::new(key).unwrap();
            keys.push(c_key);
            values.push(value);
        }
        let key_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_table_double)(
                key_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                values.as_mut_ptr(),
                key_ptrs.len() as i32,
            );
        }
    }

    /// 関数の返り値に文字列の連想配列を追加する。
    pub fn push_result_table_str<'a, T>(&self, table: T)
    where
        T: std::iter::IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in table {
            let c_key = std::ffi::CString::new(key).unwrap();
            let c_value = std::ffi::CString::new(value).unwrap();
            keys.push(c_key);
            values.push(c_value);
        }
        let key_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        let value_ptrs: Vec<*const std::os::raw::c_char> =
            values.iter().map(|v| v.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_table_string)(
                key_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                value_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                key_ptrs.len() as i32,
            );
        }
    }

    /// 関数の返り値に整数の配列を追加する。
    pub fn push_result_array_int(&self, values: &[i32]) {
        unsafe {
            ((*self.internal).push_result_array_int)(
                values.as_ptr() as *mut i32,
                values.len() as i32,
            );
        }
    }

    /// 関数の返り値に浮動小数点数の配列を追加する。
    pub fn push_result_array_float(&self, values: &[f64]) {
        unsafe {
            ((*self.internal).push_result_array_double)(
                values.as_ptr() as *mut f64,
                values.len() as i32,
            );
        }
    }

    /// 関数の返り値に文字列の配列を追加する。
    pub fn push_result_array_str(&self, values: &[&str]) {
        let c_values: Vec<std::ffi::CString> = values
            .iter()
            .map(|s| std::ffi::CString::new(*s).unwrap())
            .collect();
        let c_value_ptrs: Vec<*const std::os::raw::c_char> =
            c_values.iter().map(|s| s.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_array_string)(
                c_value_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                c_value_ptrs.len() as i32,
            );
        }
    }
}
impl From<*mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM> for ScriptModuleCallHandle {
    fn from(ptr: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM) -> Self {
        Self { internal: ptr }
    }
}

/// スクリプトモジュールの引数として受け取れる値。
///
/// # Note
///
/// このtraitはDeriveマクロを使用して実装することもできます。
/// 詳細は[`derive@FromScriptModuleParam`]のドキュメントを参照してください。
pub trait FromScriptModuleParam<'a>: Sized {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self>;
}

impl<'a> FromScriptModuleParam<'a> for i32 {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            Some(param.get_param_int(index))
        } else {
            None
        }
    }
}
impl<'a> FromScriptModuleParam<'a> for f64 {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            Some(param.get_param_float(index))
        } else {
            None
        }
    }
}
impl<'a> FromScriptModuleParam<'a> for String {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            param.get_param_str(index)
        } else {
            None
        }
    }
}

impl<'a, T> FromScriptModuleParam<'a> for Option<T>
where
    T: FromScriptModuleParam<'a>,
{
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            Some(T::from_param(param, index))
        } else {
            None
        }
    }
}

/// スクリプトモジュールの引数として渡される配列。
pub struct ScriptModuleParamArray<'a> {
    index: usize,
    ptr: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM,
    marker: std::marker::PhantomData<&'a ()>,
}

impl std::fmt::Debug for ScriptModuleParamArray<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptModuleParamArray")
            .field("index", &self.index)
            .field("len", &self.len())
            .finish()
    }
}

impl<'a> ScriptModuleParamArray<'a> {
    /// 配列の長さを返す。
    pub fn len(&self) -> usize {
        unsafe { ((*self.ptr).get_param_array_num)(self.index as i32) as usize }
    }

    /// 配列が空かどうかを返す。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 配列の要素を整数として取得する。
    pub fn get_int(&self, array_index: usize) -> i32 {
        unsafe { ((*self.ptr).get_param_array_int)(self.index as i32, array_index as i32) }
    }

    /// 配列の要素を浮動小数点数として取得する。
    pub fn get_float(&self, array_index: usize) -> f64 {
        unsafe { ((*self.ptr).get_param_array_double)(self.index as i32, array_index as i32) }
    }

    /// 配列の要素を文字列として取得する。
    pub fn get_str(&self, array_index: usize) -> Option<String> {
        unsafe {
            let c_str = ((*self.ptr).get_param_array_string)(self.index as i32, array_index as i32);
            if c_str.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }
}

impl<'a> FromScriptModuleParam<'a> for ScriptModuleParamArray<'a> {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            Some(ScriptModuleParamArray {
                index,
                ptr: param.internal,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}

/// スクリプトモジュールの引数として渡される連想配列。
#[derive(Debug)]
pub struct ScriptModuleParamTable<'a> {
    index: usize,
    ptr: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM,
    marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ScriptModuleParamTable<'a> {
    /// 連想配列の要素を整数として取得する。
    pub fn get_int(&self, key: &str) -> i32 {
        let c_key = std::ffi::CString::new(key).unwrap();
        unsafe { ((*self.ptr).get_param_table_int)(self.index as i32, c_key.as_ptr()) }
    }

    /// 連想配列の要素を浮動小数点数として取得する。
    pub fn get_float(&self, key: &str) -> f64 {
        let c_key = std::ffi::CString::new(key).unwrap();
        unsafe { ((*self.ptr).get_param_table_double)(self.index as i32, c_key.as_ptr()) }
    }

    /// 連想配列の要素を文字列として取得する。
    pub fn get_str(&self, key: &str) -> Option<String> {
        let c_key = std::ffi::CString::new(key).unwrap();
        unsafe {
            let c_str = ((*self.ptr).get_param_table_string)(self.index as i32, c_key.as_ptr());
            if c_str.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(c_str)
                        .to_string_lossy()
                        .into_owned(),
                )
            }
        }
    }
}

impl<'a> FromScriptModuleParam<'a> for ScriptModuleParamTable<'a> {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            Some(ScriptModuleParamTable {
                index,
                ptr: param.internal,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }
}

impl<'a> FromScriptModuleParam<'a> for Vec<String> {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        let array = ScriptModuleParamArray::from_param(param, index)?;
        let mut result = Vec::new();
        for i in 0..array.len() {
            result.push(array.get_str(i)?);
        }
        Some(result)
    }
}
impl<'a> FromScriptModuleParam<'a> for Vec<i32> {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        let array = ScriptModuleParamArray::from_param(param, index)?;
        let mut result = Vec::new();
        for i in 0..array.len() {
            result.push(array.get_int(i));
        }
        Some(result)
    }
}
impl<'a> FromScriptModuleParam<'a> for Vec<f64> {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        let array = ScriptModuleParamArray::from_param(param, index)?;
        let mut result = Vec::new();
        for i in 0..array.len() {
            result.push(array.get_float(i));
        }
        Some(result)
    }
}

/// 連想配列の値として使える型。
pub trait FromScriptModuleParamTable<'a>: Sized {
    fn from_param_table(param: &'a ScriptModuleParamTable, key: &str) -> Option<Self>;
}

impl<'a> FromScriptModuleParamTable<'a> for i32 {
    fn from_param_table(param: &'a ScriptModuleParamTable, key: &str) -> Option<Self> {
        Some(param.get_int(key))
    }
}
impl<'a> FromScriptModuleParamTable<'a> for f64 {
    fn from_param_table(param: &'a ScriptModuleParamTable, key: &str) -> Option<Self> {
        Some(param.get_float(key))
    }
}
impl<'a> FromScriptModuleParamTable<'a> for String {
    fn from_param_table(param: &'a ScriptModuleParamTable, key: &str) -> Option<Self> {
        param.get_str(key)
    }
}
impl<'a, T: FromScriptModuleParamTable<'a>> FromScriptModuleParamTable<'a> for Option<T> {
    fn from_param_table(param: &'a ScriptModuleParamTable, key: &str) -> Option<Self> {
        Some(T::from_param_table(param, key))
    }
}

/// 関数の戻り値として使える型。
///
/// # Note
///
/// この関数はDeriveマクロを使用して実装することもできます。
/// 詳細は[`derive@ToScriptModuleReturnValue`]のドキュメントを参照してください。
pub trait ToScriptModuleReturnValue {
    fn push_value(&self, param: &ScriptModuleCallHandle);
}

impl ToScriptModuleReturnValue for i32 {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        param.push_result_int(*self);
    }
}
impl ToScriptModuleReturnValue for f64 {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        param.push_result_float(*self);
    }
}
impl ToScriptModuleReturnValue for &str {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        param.push_result_str(self);
    }
}
impl ToScriptModuleReturnValue for String {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        param.push_result_str(self);
    }
}
impl<T: ToScriptModuleReturnValue> ToScriptModuleReturnValue for Option<T> {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        if let Some(value) = self {
            value.push_value(param);
        }
    }
}
impl<T: ToScriptModuleReturnValue, const N: usize> ToScriptModuleReturnValue for [T; N] {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        for value in self.iter() {
            value.push_value(param);
        }
    }
}
impl<T: ToScriptModuleReturnValue, E: std::error::Error> ToScriptModuleReturnValue
    for Result<T, E>
{
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        match self {
            Ok(value) => value.push_value(param),
            Err(err) => {
                param.set_error(&err.to_string());
            }
        }
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl ToScriptModuleReturnValue for Tuple {
    fn push_value(&self, param: &ScriptModuleCallHandle) {
        for_tuples!(#(
            Tuple.push_value(param);
        )*);
    }
}
