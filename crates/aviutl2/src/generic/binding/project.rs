use crate::AnyResult;

/// プロジェクトファイルにデータを保存・取得するための構造体。
pub struct ProjectFile {
    pub(crate) internal: *mut aviutl2_sys::plugin2::PROJECT_FILE,
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
    /// 文字列が見つからなかった場合はNoneを返します。
    pub fn get_param_string(&self, key: &str) -> Option<String> {
        unsafe {
            let raw_str = ((*self.internal).get_param_string)(key.as_ptr() as _);
            if raw_str.is_null() {
                return None;
            }
            Some(
                std::ffi::CStr::from_ptr(raw_str)
                    .to_string_lossy()
                    .to_string(),
            )
        }
    }

    /// プロジェクトに保存されているバイナリデータを取得します。
    ///
    /// # Errors
    ///
    /// - `data` の長さが保存されているデータの長さと一致しない場合、失敗します。
    /// - 指定されたキーに対応するデータが存在しない場合、失敗します。
    pub fn get_param_binary(&self, key: &str, data: &mut [u8]) -> AnyResult<()> {
        let success = unsafe {
            ((*self.internal).get_param_binary)(
                key.as_ptr() as _,
                data.as_mut_ptr() as _,
                data.len() as _,
            )
        };
        anyhow::ensure!(success, "failed to get binary data for key {}", key);
        Ok(())
    }

    /// プロジェクトに文字列を保存します。
    ///
    /// # Errors
    ///
    /// key、valueにヌル文字が含まれている場合、失敗します。
    pub fn set_param_string(&mut self, key: &str, value: &str) -> AnyResult<()> {
        let key_cstr = std::ffi::CString::new(key)?;
        let value_cstr = std::ffi::CString::new(value)?;
        unsafe {
            ((*self.internal).set_param_string)(key_cstr.as_ptr() as _, value_cstr.as_ptr() as _);
        }
        Ok(())
    }

    /// プロジェクトにバイナリデータを保存します。
    ///
    /// # Errors
    ///
    /// `data` の長さが4096バイトを超える場合、失敗します。
    pub fn set_param_binary(&mut self, key: &str, data: &[u8]) -> AnyResult<()> {
        anyhow::ensure!(data.len() <= 4096, "data length exceeds 4096 bytes");
        unsafe {
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
        let num_bytes = bytes.len() as u32;
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
            .get_param_string(key)
            .ok_or_else(|| anyhow::anyhow!("no data found for key {}", key))?;
        let header_prefix = format!("{NAMESPACE}:serde-zstd-v1:");
        anyhow::ensure!(
            header.starts_with(&header_prefix),
            "invalid header for key {}",
            key
        );
        let num_bytes: usize = header[header_prefix.len()..].parse()?;
        let mut bytes = Vec::with_capacity(num_bytes);
        let mut read_bytes = 0;
        for i in 0.. {
            let chunk_key = format!("{NAMESPACE}:serde-zstd-v1:chunk:{}:{}", key, i);
            let mut chunk = vec![0u8; 4096];
            match self.get_param_binary(&chunk_key, &mut chunk) {
                Ok(()) => {
                    let to_read = std::cmp::min(4096, num_bytes - read_bytes);
                    bytes.extend_from_slice(&chunk[..to_read]);
                    read_bytes += to_read;
                    if read_bytes >= num_bytes {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let decompressed_bytes = zstd::decode_all(&bytes[..])?;
        let value: T = rmp_serde::from_slice(&decompressed_bytes)?;
        Ok(value)
    }
}
