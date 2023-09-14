use crate::{core::types::*, ffi::*};

pub(crate) unsafe fn get_logical_cpu_count() -> u32 {
    let system_info: &mut SystemInfo = &mut core::mem::zeroed::<SystemInfo>();

    GetSystemInfo(system_info);

    system_info.dw_number_of_processors
}

pub(crate) unsafe fn get_dmi_info() -> crate::Result<DmiInfo> {
    let signature: [u8; 4] = *b"RSMB";

    let signature: u32 = ((signature[0] as u32) << 24)
        | ((signature[1] as u32) << 16)
        | ((signature[2] as u32) << 8)
        | (signature[3] as u32);

    let mut return_length: u32 = GetSystemFirmwareTable(signature, 0, core::ptr::null_mut(), 0);

    let mut buffer: Vec<u8> = vec![0; return_length as usize];

    return_length = GetSystemFirmwareTable(signature, 0, buffer.as_mut_ptr(), return_length);

    if return_length > return_length {
        return Err("The function GetSystemFirmwareTable failed".to_string());
    }

    let get_string_by_dmi: fn(*const DmiHeader, u8) -> crate::Result<String> =
        |dm_header: *const DmiHeader, mut index: u8| -> crate::Result<String> {
            let get_c_str_len: fn(*const i8) -> usize = |cstr: *const i8| -> usize {
                let mut len: usize = 0;

                while cstr.add(len).read() != 0 {
                    len += 1;
                }

                len
            };

            let base_address: *const i8 = dm_header.cast::<i8>();

            if index == 0 {
                return Err("Invalid index".to_string());
            }

            let mut base_address: *const i8 = base_address.add(dm_header.read().length as usize);

            while index > 1 && base_address.read() != 0 {
                base_address = base_address.add(get_c_str_len(base_address));

                base_address = base_address.add(1);

                index -= 1;
            }

            if base_address.read() == 0 {
                return Err("Invalid base address".to_string());
            }

            let len: usize = get_c_str_len(base_address);

            let bp_vec: &[u8] = std::slice::from_raw_parts(base_address.cast::<u8>(), len);

            Ok(String::from_utf8_lossy(bp_vec).to_string())
        };

    let smb: RawSMBIOSData = RawSMBIOSData {
        used20_calling_method: buffer[0],
        smbiosmajor_version: buffer[1],
        smbiosminor_version: buffer[2],
        dmi_revision: buffer[3],
        length: u32::from_ne_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
        smbiostable_data: buffer[8..].to_vec(),
    };

    let mut dmi_info: DmiInfo = DmiInfo::default();

    let mut uuid: [u8; 16] = [0; 16];

    let mut data: *const u8 = smb.smbiostable_data.as_ptr();

    let mut once_flag: bool = false;

    while data < smb.smbiostable_data.as_ptr().add(smb.length as usize) {
        let h: *const DmiHeader = data.cast();

        if h.read().length < 4 {
            break;
        }

        if h.read().ctype == 0 && once_flag == false {
            if let Ok(bios_vendor) = get_string_by_dmi(h, data.offset(0x4).read()) {
                dmi_info.bios_vendor = bios_vendor;
            }

            if let Ok(bios_version) = get_string_by_dmi(h, data.offset(0x5).read()) {
                dmi_info.bios_version = bios_version;
            }

            if let Ok(bios_release_date) = get_string_by_dmi(h, data.offset(0x8).read()) {
                dmi_info.bios_release_date = bios_release_date;
            }

            if data.offset(0x16).read() != 0xFF && data.offset(0x17).read() != 0xFF {
                dmi_info.bios_embedded_controller_firmware_version =
                    format!("{}.{}", data.offset(0x16).read(), data.offset(0x17).read());
            }

            once_flag = true;
        }

        if h.read().ctype == 0x01 && h.read().length >= 0x19 {
            if let Ok(manufacturer) = get_string_by_dmi(h, data.offset(0x4).read()) {
                dmi_info.system_manufacturer = manufacturer;
            }

            if let Ok(product) = get_string_by_dmi(h, data.offset(0x5).read()) {
                dmi_info.system_product = product;
            }

            if let Ok(version) = get_string_by_dmi(h, data.offset(0x6).read()) {
                dmi_info.system_version = version;
            }

            if let Ok(serial_number) = get_string_by_dmi(h, data.offset(0x7).read()) {
                dmi_info.system_serial_number = serial_number;
            }

            if let Ok(sku_number) = get_string_by_dmi(h, data.offset(0x19).read()) {
                dmi_info.system_sku_number = sku_number;
            }

            if let Ok(family) = get_string_by_dmi(h, data.offset(0x1A).read()) {
                dmi_info.system_family = family;
            }

            data = data.add(0x8);

            let mut all_zero: bool = true;

            let mut all_one: bool = true;

            let mut i: isize = 0;

            while i < 16 && (all_zero || all_one) {
                if data.offset(i).read() != 0x00 {
                    all_zero = false;
                }

                if data.offset(i).read() != 0xFF {
                    all_one = false;
                }

                i += 1;
            }

            if !all_zero && !all_one {
                for i in 0..4 {
                    uuid[i] = data.add(i).read();
                }

                uuid[5] = data.offset(5).read();

                uuid[4] = data.offset(4).read();

                uuid[7] = data.offset(7).read();

                uuid[6] = data.offset(6).read();

                for j in 8..16 {
                    uuid[j] = data.add(j).read();
                }

                let mut uuid_string: String = String::new();

                for i in 0..16 {
                    uuid_string.push_str(format!("{:02X}", uuid[i]).as_str());

                    if (i + 1) % 4 == 0 && i != 15 {
                        uuid_string.push('-');
                    }
                }

                let mut guid: [u8; 16] = uuid;

                for (i, j) in (0..4).zip((0..4).rev()) {
                    guid[i] = uuid[j];
                }

                guid[4] = uuid[5];

                guid[5] = uuid[4];

                guid[6] = uuid[7];

                guid[7] = uuid[6];

                dmi_info.system_uuid = (uuid, uuid_string);

                let mut guid_string: String = String::new();

                for i in 0..16 {
                    guid_string.push_str(format!("{:02X}", guid[i]).as_str());

                    if i == 3 {
                        guid_string.push('-');
                    }

                    if i % 2 == 1 && i < 10 && i > 4 {
                        guid_string.push('-');
                    }
                }

                dmi_info.system_guid = (guid, guid_string);
            }

            break;
        }

        let mut next: *const u8 = data.add(h.read().length as usize);

        while next < smb.smbiostable_data.as_ptr().add(smb.length as usize)
            && (next.offset(0).read() != 0 || next.offset(1).read() != 0)
        {
            next = next.add(1);
        }

        data = next.add(2);
    }

    Ok(dmi_info)
}