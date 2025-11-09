/// 関数の引数・返り値を扱うための型とトレイト。
#[derive(Debug)]
pub struct ScriptModuleCallHandle {
    pub(crate) internal: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM,
}

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

    /// 引数を取得する。
    pub fn get_param<'a, T: FromScriptModuleParam<'a>>(&'a self, index: usize) -> Option<T> {
        T::from_param(self, index)
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

    /// 引数をブール値として取得する。
    ///
    /// # Note
    ///
    /// 引数を取得できない場合は`false`を返します。
    pub fn get_param_boolean(&self, index: usize) -> bool {
        unsafe { ((*self.internal).get_param_boolean)(index as i32) }
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
    pub fn set_error(&mut self, message: &str) -> Result<(), std::ffi::NulError> {
        let c_message = std::ffi::CString::new(message)?;
        unsafe {
            ((*self.internal).set_error)(c_message.as_ptr());
        }
        Ok(())
    }

    /// 関数の返り値を追加する。
    pub fn push_result<T: IntoScriptModuleReturnValue>(
        &mut self,
        value: T,
    ) -> crate::AnyResult<()> {
        value.push_into(self)
    }

    /// 関数の返り値に整数を追加する。
    pub fn push_result_int(&mut self, value: i32) {
        unsafe {
            ((*self.internal).push_result_int)(value);
        }
    }

    /// 関数の返り値に浮動小数点数を追加する。
    pub fn push_result_float(&mut self, value: f64) {
        unsafe {
            ((*self.internal).push_result_double)(value);
        }
    }

    /// 関数の返り値に文字列を追加する。
    pub fn push_result_str(&mut self, value: &str) -> crate::AnyResult<()> {
        let c_value = std::ffi::CString::new(value)?;
        unsafe {
            ((*self.internal).push_result_string)(c_value.as_ptr());
        }
        Ok(())
    }

    /// 関数の返り値に整数の連想配列を追加する。
    pub fn push_result_table_int<'a, T>(&mut self, table: T) -> crate::AnyResult<()>
    where
        T: std::iter::IntoIterator<Item = (&'a str, i32)>,
    {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in table {
            let c_key = std::ffi::CString::new(key)?;
            keys.push(c_key);
            values.push(value);
        }
        let key_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_table_int)(
                key_ptrs.as_ptr(),
                values.as_ptr(),
                key_ptrs.len() as i32,
            );
        }
        Ok(())
    }

    /// 関数の返り値に浮動小数点数の連想配列を追加する。
    pub fn push_result_table_float<'a, T>(&mut self, table: T) -> crate::AnyResult<()>
    where
        T: std::iter::IntoIterator<Item = (&'a str, f64)>,
    {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in table {
            let c_key = std::ffi::CString::new(key)?;
            keys.push(c_key);
            values.push(value);
        }
        let key_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_table_double)(
                key_ptrs.as_ptr(),
                values.as_ptr(),
                key_ptrs.len() as i32,
            );
        }
        Ok(())
    }

    /// 関数の返り値に文字列の連想配列を追加する。
    pub fn push_result_table_str<'a, T>(&mut self, table: T) -> crate::AnyResult<()>
    where
        T: std::iter::IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in table {
            let c_key = std::ffi::CString::new(key)?;
            let c_value = std::ffi::CString::new(value)?;
            keys.push(c_key);
            values.push(c_value);
        }
        anyhow::ensure!(
            keys.len() <= i32::MAX as usize,
            "Table size exceeds i32::MAX"
        );
        let key_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        let value_ptrs: Vec<*const std::os::raw::c_char> =
            values.iter().map(|v| v.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_table_string)(
                key_ptrs.as_ptr(),
                value_ptrs.as_ptr(),
                key_ptrs.len() as i32,
            );
        }
        Ok(())
    }

    /// 関数の返り値に整数の配列を追加する。
    pub fn push_result_array_int(&mut self, values: &[i32]) -> crate::AnyResult<()> {
        anyhow::ensure!(
            values.len() <= i32::MAX as usize,
            "Array length exceeds i32::MAX"
        );
        unsafe {
            ((*self.internal).push_result_array_int)(values.as_ptr(), values.len() as i32);
        }
        Ok(())
    }

    /// 関数の返り値に浮動小数点数の配列を追加する。
    pub fn push_result_array_float(&mut self, values: &[f64]) -> crate::AnyResult<()> {
        anyhow::ensure!(
            values.len() <= i32::MAX as usize,
            "Array length exceeds i32::MAX"
        );
        unsafe {
            ((*self.internal).push_result_array_double)(values.as_ptr(), values.len() as i32);
        }
        Ok(())
    }

    /// 関数の返り値に文字列の配列を追加する。
    pub fn push_result_array_str(&mut self, values: &[&str]) -> crate::AnyResult<()> {
        let c_values: Vec<std::ffi::CString> = values
            .iter()
            .map(|s| std::ffi::CString::new(*s))
            .collect::<Result<_, _>>()?;
        anyhow::ensure!(
            c_values.len() <= i32::MAX as usize,
            "Array length exceeds i32::MAX"
        );
        let c_value_ptrs: Vec<*const std::os::raw::c_char> =
            c_values.iter().map(|s| s.as_ptr()).collect();
        unsafe {
            ((*self.internal).push_result_array_string)(
                c_value_ptrs.as_ptr(),
                c_value_ptrs.len() as i32,
            );
        }
        Ok(())
    }

    /// 関数の返り値にブール値を追加する。
    pub fn push_result_boolean(&mut self, value: bool) {
        unsafe {
            ((*self.internal).push_result_boolean)(value);
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
pub use aviutl2_macros::FromScriptModuleParam;

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
impl<'a> FromScriptModuleParam<'a> for bool {
    fn from_param(param: &'a ScriptModuleCallHandle, index: usize) -> Option<Self> {
        if index < param.len() {
            Some(param.get_param_boolean(index))
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

/// スクリプトモジュールの関数の戻り値の型を表す列挙型。
#[derive(Debug, Clone)]
pub enum ScriptModuleReturnValue {
    Int(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    StringArray(Vec<String>),
    IntArray(Vec<i32>),
    FloatArray(Vec<f64>),
    IntTable(std::collections::HashMap<String, i32>),
    FloatTable(std::collections::HashMap<String, f64>),
    StringTable(std::collections::HashMap<String, String>),
}

/// 関数の戻り値として使える型。
///
/// # Note
///
/// この関数はDeriveマクロを使用して実装することもできます。
/// 詳細は[`derive@IntoScriptModuleReturnValue`]のドキュメントを参照してください。
pub trait IntoScriptModuleReturnValue {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>>;
    fn push_into(self, param: &mut ScriptModuleCallHandle) -> crate::AnyResult<()>
    where
        Self: std::marker::Sized,
    {
        for value in self.into_return_values()? {
            match value {
                ScriptModuleReturnValue::Int(v) => {
                    param.push_result_int(v);
                }
                ScriptModuleReturnValue::Float(v) => {
                    param.push_result_float(v);
                }
                ScriptModuleReturnValue::String(v) => {
                    param.push_result_str(&v)?;
                }
                ScriptModuleReturnValue::Boolean(v) => {
                    param.push_result_boolean(v);
                }
                ScriptModuleReturnValue::StringArray(v) => {
                    let strs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
                    param.push_result_array_str(&strs)?
                }
                ScriptModuleReturnValue::IntArray(v) => param.push_result_array_int(&v)?,
                ScriptModuleReturnValue::FloatArray(v) => param.push_result_array_float(&v)?,
                ScriptModuleReturnValue::IntTable(v) => {
                    let table = v.iter().map(|(k, v)| (k.as_str(), *v));
                    param.push_result_table_int(table)?;
                }
                ScriptModuleReturnValue::FloatTable(v) => {
                    let table = v.iter().map(|(k, v)| (k.as_str(), *v));
                    param.push_result_table_float(table)?;
                }
                ScriptModuleReturnValue::StringTable(v) => {
                    let table = v.iter().map(|(k, v)| (k.as_str(), v.as_str()));
                    param.push_result_table_str(table)?;
                }
            };
        }
        Ok(())
    }
}
pub use aviutl2_macros::IntoScriptModuleReturnValue;

impl IntoScriptModuleReturnValue for i32 {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::Int(self)])
    }
}
#[duplicate::duplicate_item(
    Integer;
    [i8];
    [i16];
    [i64];
    [i128];
    [isize];
    [u8];
    [u16];
    [u32];
    [u64];
    [u128];
    [usize];
)]
impl IntoScriptModuleReturnValue for Integer {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        #[allow(clippy::unnecessary_fallible_conversions)]
        Ok(vec![ScriptModuleReturnValue::Int(self.try_into()?)])
    }
}
impl IntoScriptModuleReturnValue for f64 {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::Float(self)])
    }
}
impl IntoScriptModuleReturnValue for f32 {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::Float(self as f64)])
    }
}
impl IntoScriptModuleReturnValue for bool {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::Boolean(self)])
    }
}
impl IntoScriptModuleReturnValue for &str {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::String(self.to_string())])
    }
}
impl IntoScriptModuleReturnValue for String {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::String(self)])
    }
}

impl IntoScriptModuleReturnValue for ScriptModuleReturnValue {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![self])
    }
}

impl IntoScriptModuleReturnValue for Vec<ScriptModuleReturnValue> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(self)
    }
}

impl<T: IntoScriptModuleReturnValue> IntoScriptModuleReturnValue for Option<T> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        if let Some(value) = self {
            value.into_return_values()
        } else {
            Ok(Vec::new())
        }
    }
}
impl<T: IntoScriptModuleReturnValue, const N: usize> IntoScriptModuleReturnValue for [T; N] {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        let mut out = Vec::new();
        for value in self.into_iter() {
            out.extend(value.into_return_values()?);
        }
        Ok(out)
    }
}
impl<T: IntoScriptModuleReturnValue, E> IntoScriptModuleReturnValue for Result<T, E>
where
    Box<dyn std::error::Error>: From<E>,
{
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        match self {
            Ok(value) => value.into_return_values(),
            Err(_) => Ok(Vec::new()),
        }
    }
    fn push_into(self, param: &mut ScriptModuleCallHandle) -> crate::AnyResult<()> {
        match self {
            Ok(value) => {
                value.push_into(param)?;
            }
            Err(err) => {
                let _ = param.set_error(&(Box::<dyn std::error::Error>::from(err).to_string()));
            }
        }
        Ok(())
    }
}

impl IntoScriptModuleReturnValue for () {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(Vec::new())
    }
}

#[impl_trait_for_tuples::impl_for_tuples(1, 10)]
impl IntoScriptModuleReturnValue for Tuple {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        let mut vec = Vec::new();
        for_tuples!(#(
            vec.extend(Tuple.into_return_values()?);
        )*);
        Ok(vec)
    }
}

impl IntoScriptModuleReturnValue for Vec<String> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::StringArray(self)])
    }
}
impl IntoScriptModuleReturnValue for Vec<&str> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::StringArray(
            self.iter().map(|s| s.to_string()).collect(),
        )])
    }
}
impl IntoScriptModuleReturnValue for Vec<i32> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::IntArray(self)])
    }
}
impl IntoScriptModuleReturnValue for Vec<f64> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::FloatArray(self)])
    }
}
impl<T> IntoScriptModuleReturnValue for &[T]
where
    T: IntoScriptModuleReturnValue + Clone,
{
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        let mut vec = Vec::new();
        for item in self.iter() {
            vec.extend(item.clone().into_return_values()?);
        }
        Ok(vec)
    }
}

impl IntoScriptModuleReturnValue for std::collections::HashMap<String, i32> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::IntTable(self)])
    }
}
impl IntoScriptModuleReturnValue for std::collections::HashMap<String, f64> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::FloatTable(self)])
    }
}
impl IntoScriptModuleReturnValue for std::collections::HashMap<String, String> {
    fn into_return_values(self) -> crate::AnyResult<Vec<ScriptModuleReturnValue>> {
        Ok(vec![ScriptModuleReturnValue::StringTable(self)])
    }
}

#[doc(hidden)]
pub mod table_converter {
    pub trait ToOptionalTableEntry {
        type Value;
        fn to_optional(&self) -> Option<Self::Value>;
    }

    impl<T: Clone> ToOptionalTableEntry for Option<T> {
        type Value = T;
        fn to_optional(&self) -> Option<Self::Value> {
            self.clone()
        }
    }
    impl ToOptionalTableEntry for i32 {
        type Value = i32;
        fn to_optional(&self) -> Option<Self::Value> {
            Some(*self)
        }
    }
    impl ToOptionalTableEntry for f64 {
        type Value = f64;
        fn to_optional(&self) -> Option<Self::Value> {
            Some(*self)
        }
    }
    impl ToOptionalTableEntry for String {
        type Value = String;
        fn to_optional(&self) -> Option<Self::Value> {
            Some(self.clone())
        }
    }
}
