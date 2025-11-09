/// プロジェクトファイルにデータを保存・取得するための構造体。
pub struct ProjectFile {
    pub(crate) internal: *mut aviutl2_sys::plugin2::PROJECT_FILE,
}

/// プロジェクトファイルのデータ取得・保存に関するエラー。
#[derive(thiserror::Error, Debug)]
pub enum ProjectFileError {
    #[error("key contains null byte: {0}")]
    KeyContainsNull(std::ffi::NulError),
    #[error("data retrieval failed for key {0}")]
    RetrievalFailed(String),
    #[error("data length exceeds 4096 bytes, got {0} bytes")]
    DataTooLarge(usize),
    #[error("value contains null byte: {0}")]
    ValueContainsNull(std::ffi::NulError),
}

impl ProjectFile {
    /// 生ポインタから`ProjectFile`を作成します。
    ///
    /// # Safety
    ///
    /// - `raw`は有効な`PROJECT_FILE`ポインタである必要があります。
    pub unsafe fn from_raw(raw: *mut aviutl2_sys::plugin2::PROJECT_FILE) -> Self {
        Self { internal: raw }
    }

    /// プロジェクトに保存されている文字列を取得します。
    ///
    /// # Errors
    ///
    /// - `key`にヌル文字が含まれている場合、失敗します。
    /// - 文字列が見つからなかった場合は失敗します。
    pub fn get_param_string(&self, key: &str) -> Result<String, ProjectFileError> {
        let c_key = std::ffi::CString::new(key).map_err(ProjectFileError::KeyContainsNull)?;
        unsafe {
            let raw_str = ((*self.internal).get_param_string)(c_key.as_ptr() as _);
            if raw_str.is_null() {
                return Err(ProjectFileError::RetrievalFailed(key.to_string()));
            }
            Ok(std::ffi::CStr::from_ptr(raw_str)
                .to_string_lossy()
                .into_owned())
        }
    }

    /// プロジェクトに保存されているバイナリデータを取得します。
    ///
    /// # Errors
    ///
    /// - `key`にヌル文字が含まれている場合、失敗します。
    /// - `data` の長さが保存されているデータの長さと一致しない場合、失敗します。
    /// - 指定されたキーに対応するデータが存在しない場合、失敗します。
    pub fn get_param_binary(&self, key: &str, data: &mut [u8]) -> Result<(), ProjectFileError> {
        let success = unsafe {
            let key = std::ffi::CString::new(key).map_err(ProjectFileError::KeyContainsNull)?;
            ((*self.internal).get_param_binary)(
                key.as_ptr() as _,
                data.as_mut_ptr() as _,
                data.len() as _,
            )
        };
        if !success {
            return Err(ProjectFileError::RetrievalFailed(key.to_string()));
        }
        Ok(())
    }

    /// プロジェクトに文字列を保存します。
    ///
    /// # Errors
    ///
    /// key、valueにヌル文字が含まれている場合、失敗します。
    pub fn set_param_string(&mut self, key: &str, value: &str) -> Result<(), ProjectFileError> {
        let key_cstr = std::ffi::CString::new(key).map_err(ProjectFileError::KeyContainsNull)?;
        let value_cstr =
            std::ffi::CString::new(value).map_err(ProjectFileError::ValueContainsNull)?;
        unsafe {
            ((*self.internal).set_param_string)(key_cstr.as_ptr() as _, value_cstr.as_ptr() as _);
        }
        Ok(())
    }

    /// プロジェクトにバイナリデータを保存します。
    ///
    /// # Errors
    ///
    /// - `data` の長さが4096バイトを超える場合、失敗します。
    /// - `key`にヌル文字が含まれている場合、失敗します。
    pub fn set_param_binary(&mut self, key: &str, data: &[u8]) -> Result<(), ProjectFileError> {
        if data.len() > 4096 {
            return Err(ProjectFileError::DataTooLarge(data.len()));
        }
        unsafe {
            let key = std::ffi::CString::new(key).map_err(ProjectFileError::KeyContainsNull)?;
            ((*self.internal).set_param_binary)(
                key.as_ptr() as _,
                data.as_ptr() as _,
                data.len() as _,
            );
        }
        Ok(())
    }

    /// プロジェクトに保存されているデータをすべて削除します。
    pub fn clear_params(&mut self) {
        unsafe { ((*self.internal).clear_params)() }
    }
}

#[cfg(feature = "serde")]
static NAMESPACE: &str = "--aviutl2-rs";

#[cfg(feature = "serde")]
impl ProjectFile {
    /// プロジェクトにデータをシリアライズして保存します。
    ///
    /// # Note
    ///
    /// 今現在の実装ではデータはMessagePackにシリアライズされた後にZstdで圧縮されています。
    ///
    /// # Errors
    ///
    /// - シリアライズに失敗した場合。
    /// - 圧縮に失敗した場合。
    pub fn serialize<T: serde::Serialize>(&mut self, key: &str, value: &T) -> crate::AnyResult<()> {
        let bytes = rmp_serde::to_vec_named(value)?;
        let bytes = zstd::encode_all(&bytes[..], 0)?;
        let num_bytes = bytes.len();
        self.set_param_string(key, &format!("{NAMESPACE}:serde-zstd-v1:{}", num_bytes))?;
        for (i, chunk) in bytes.chunks(4096).enumerate() {
            let chunk_key = format!("{NAMESPACE}:serde-zstd-v1:chunk:{}:{}", key, i);
            self.set_param_binary(&chunk_key, chunk)?;
        }
        Ok(())
    }

    /// プロジェクトからデータをデシリアライズして取得します。
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self, key: &str) -> crate::AnyResult<T> {
        let header = self
            .get_param_string(key)?
            .ok_or_else(|| anyhow::anyhow!("no data found for key {}", key))?;
        let header_prefix = format!("{NAMESPACE}:serde-zstd-v1:");
        let num_bytes = header
            .strip_prefix(&header_prefix)
            .ok_or_else(|| anyhow::anyhow!("invalid header for key {}", key))?;
        let num_bytes: usize = num_bytes.parse()?;
        if num_bytes == 0 {
            anyhow::bail!("invalid data length 0 for key {}", key);
        }
        let mut bytes = Vec::with_capacity(num_bytes);
        let mut read_bytes = 0;
        let mut chunk = vec![0u8; 4096];
        for i in 0.. {
            let chunk_key = format!("{NAMESPACE}:serde-zstd-v1:chunk:{}:{}", key, i);
            let to_read = std::cmp::min(4096, num_bytes - read_bytes);
            chunk.resize(to_read, 0);
            match self.get_param_binary(&chunk_key, &mut chunk) {
                Ok(()) => {
                    bytes.extend_from_slice(&chunk);
                    read_bytes += to_read;
                    if read_bytes >= num_bytes {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        anyhow::ensure!(
            read_bytes == num_bytes,
            "incomplete data for key {}, expected {} bytes, got {} bytes",
            key,
            num_bytes,
            read_bytes
        );
        let decompressed_bytes = zstd::decode_all(&bytes[..])?;
        let value: T = rmp_serde::from_slice(&decompressed_bytes)?;
        Ok(value)
    }
}
