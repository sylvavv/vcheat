use crate::*;

pub fn get_all_process_modules_info(
    process_id: u32,
    read_module_data: bool,
) -> Result<Vec<ModuleInfo>> {
    unsafe {
        let snapshot_handle = CreateToolhelp32Snapshot(0x8 | 0x10, process_id);

        if snapshot_handle.is_null() {
            return Err(format!(
                "CreateToolhelp32Snapshot failed with return value: {snapshot_handle:X?}"
            ));
        }

        let module_entry = &mut core::mem::zeroed() as *mut ModuleEntry32W;
        (*module_entry).dw_size = core::mem::size_of::<ModuleEntry32W>() as u32;

        let result = Module32FirstW(snapshot_handle, module_entry);

        if result == 0 {
            CloseHandle(snapshot_handle);
            return Err(format!(
                "Module32FirstW failed with return value: {result:X}"
            ));
        }

        let mut module_entry_array = Vec::<ModuleEntry32W>::new();

        module_entry_array.push(module_entry.read());

        while Module32NextW(snapshot_handle, module_entry) != 0 {
            module_entry_array.push(module_entry.read());
        }

        if !snapshot_handle.is_null() {
            close_handle(snapshot_handle)?
        }

        let mut module_info_array = Vec::<ModuleInfo>::new();

        let mut process_handle: *mut core::ffi::c_void = core::ptr::null_mut();

        if read_module_data {
            process_handle = get_process_handle(process_id)?
        }

        for m in module_entry_array {
            module_info_array.push(ModuleInfo {
                process_id: m.th32_process_id,
                module_address: m.mod_base_addr,
                module_size: m.mod_base_size,
                module_handle: m.h_module,
                module_name: core::ffi::CStr::from_ptr(
                    String::from_utf16_lossy(m.sz_module.as_ref())
                        .as_ptr()
                        .cast(),
                )
                .to_string_lossy()
                .to_string(),
                module_path: core::ffi::CStr::from_ptr(
                    String::from_utf16_lossy(m.sz_exe_path.as_ref())
                        .as_ptr()
                        .cast(),
                )
                .to_string_lossy()
                .to_string(),
                module_data: if read_module_data {
                    Some(memory::read_memory(
                        process_handle,
                        m.mod_base_addr.cast(),
                        m.mod_base_size as usize,
                    )?)
                } else {
                    None
                },
            })
        }

        if !process_handle.is_null() {
            close_handle(process_handle)?;
        }

        Ok(module_info_array)
    }
}
