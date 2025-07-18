pub struct NamedPipe {
    handle: Option<windows::Win32::Foundation::HANDLE>,
}
unsafe impl Send for NamedPipe {}
unsafe impl Sync for NamedPipe {}

impl NamedPipe {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let handle = unsafe {
            windows::Win32::System::Pipes::CreateNamedPipeW(
                &windows::core::HSTRING::from(name),
                windows::Win32::Storage::FileSystem::PIPE_ACCESS_OUTBOUND,
                windows::Win32::System::Pipes::PIPE_TYPE_BYTE,
                1,
                0,
                0,
                0,
                None,
            )
        };
        if handle.is_invalid() {
            return Err(anyhow::anyhow!("Failed to create named pipe: {}", unsafe {
                windows::Win32::Foundation::GetLastError()
                    .to_hresult()
                    .message()
            }));
        }
        Ok(NamedPipe {
            handle: Some(handle),
        })
    }

    pub fn connect(mut self) -> anyhow::Result<PipeWriter> {
        if let Some(handle) = self.handle.take() {
            PipeWriter::new(handle)
        } else {
            Err(anyhow::anyhow!("Named pipe handle is not available"))
        }
    }
}

impl Drop for NamedPipe {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
            }
        }
    }
}

pub struct PipeWriter {
    handle: windows::Win32::Foundation::HANDLE,
}

impl PipeWriter {
    fn new(handle: windows::Win32::Foundation::HANDLE) -> anyhow::Result<Self> {
        unsafe {
            if windows::Win32::System::Pipes::ConnectNamedPipe(handle, None).is_err() {
                return Err(anyhow::anyhow!(
                    "Failed to connect named pipe: {}",
                    windows::Win32::Foundation::GetLastError()
                        .to_hresult()
                        .message()
                ));
            }
        }
        Ok(PipeWriter { handle })
    }
}

impl std::io::Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut bytes_written = 0;
        unsafe {
            if windows::Win32::Storage::FileSystem::WriteFile(
                self.handle,
                Some(buf),
                Some(&mut bytes_written),
                None,
            )
            .is_err()
            {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for PipeWriter {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Pipes::DisconnectNamedPipe(self.handle);
            let _ = windows::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}
