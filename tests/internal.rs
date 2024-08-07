#[test]
fn get_mod_info() {
    unsafe {
        let mi = vcheat::internal::get_mod_info("").unwrap();
        println!("{:#?}", mi);
    }
}

#[test]
fn read_write_mem() {
    unsafe {
        let proc_handle = vcheat::internal::get_proc_handle();

        let mi = vcheat::internal::get_mod_info("").unwrap();

        vcheat::internal::protect_mem(
            mi.addr,
            mi.size as usize,
            vcheat::types::mem_protect::EXECUTE_READ_WRITE,
        )
        .unwrap();

        let mod_data = vcheat::read_mem(proc_handle, mi.addr, mi.size as usize).unwrap();

        let mut mod_data1 = vec![0u8; mi.size as usize];

        vcheat::read_mem_t(
            proc_handle,
            mi.addr,
            mod_data1.as_mut_ptr(),
            mi.size as usize,
        )
        .unwrap();

        vcheat::write_mem(proc_handle, mi.addr, mod_data).unwrap();

        vcheat::write_mem_t(proc_handle, mi.addr, mod_data1.as_ptr(), mi.size as usize).unwrap();
    }
}

#[test]
fn load_free_dll() {
    unsafe {
        let mod_handle = vcheat::internal::load_dll("dinput8.dll").unwrap();
        vcheat::internal::free_dll(mod_handle).unwrap();
    }
}

#[test]
fn alloc_free_mem() {
    unsafe {
        let alloc = vcheat::internal::alloc_mem(
            ::core::ptr::null(),
            0x1000,
            vcheat::types::mem_alloc::COMMIT,
            vcheat::types::mem_protect::READ_WRITE,
        )
        .unwrap();

        let proc_handle = vcheat::internal::get_proc_handle();

        vcheat::write_mem(proc_handle, alloc, &[0xD2_u8, 0x04]).unwrap();

        let buf = vcheat::read_mem(proc_handle, alloc, 4).unwrap();

        vcheat::internal::free_mem(alloc, 0, vcheat::types::mem_free::RELEASE).unwrap();

        let bytes = [buf[0], buf[1], buf[2], buf[3]];

        let num = i32::from_le_bytes(bytes);

        assert_eq!(num, 1234);
    }
}

fn _read_multi_pointer() {
    unsafe {
        let _final_ptr = vcheat::internal::read_multi_pointer(
            0x123456 as *const ::core::ffi::c_void,
            &[0xAB, 0xCD, 0x10, 0x20],
        )
        .unwrap();
    }
}

#[test]
fn query_mem_type() {
    unsafe {
        let mi = vcheat::internal::get_mod_info("").unwrap();

        println!(
            "{}",
            vcheat::internal::check_mem_protect(mi.addr, vcheat::types::MemQueryProtect::WRITE)
                .unwrap()
        );
    }
}
