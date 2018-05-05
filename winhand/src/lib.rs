#[macro_use]
extern crate failure;
extern crate uuid;
extern crate widestring;
extern crate winapi;

use failure::Error;
use std::path::Path;
use std::ptr::null_mut;
use widestring::WideCString;
use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::ERROR_MORE_DATA;
use winapi::um::restartmanager;

#[derive(Debug)]
pub struct Process {
    pub process_id: u32,
    pub start_time: (u32, u32),
    pub app_name: String,
    pub service_short_name: String,
    pub application_type: restartmanager::RM_APP_TYPE,
    pub app_status: u32,
}

pub fn get_procs_using_path<P: AsRef<Path>>(path: P) -> Result<Vec<Process>, Error> {
    let mut session_handle: DWORD = 0;
    let key = uuid::Uuid::new_v4().simple().to_string();
    let key = WideCString::from_str(key)?;
    // as *mut DWORD
    unsafe {
        let res = restartmanager::RmStartSession(&mut session_handle, 0, key.into_raw());
        if res != 0 {
            return Err(format_err!("Failed to start session: {}.", res));
        }
        let wide_path = WideCString::from_str(path.as_ref().to_str().expect("xxx"))?;
        let mut resources = vec![wide_path.as_ptr()];

        let res = if restartmanager::RmRegisterResources(
            session_handle,
            1,
            resources.as_mut_ptr(),
            0,
            null_mut(),
            0,
            null_mut(),
        ) != 0
        {
            Err(failure::err_msg("Could not register resource."))
        } else {
            let mut n_proc_info_needed = 0;
            let mut n_proc_info = 0;
            let mut reboot_reasons = restartmanager::RmRebootReasonNone;

            // Determine how much memory we need.
            let res = restartmanager::RmGetList(
                session_handle,
                &mut n_proc_info_needed,
                &mut n_proc_info,
                null_mut(),
                &mut reboot_reasons,
            );
            if res == 0 {
                Ok(vec![])
            } else if res != ERROR_MORE_DATA {
                Err(format_err!("Unexpected error {:?}", res))
            } else {
                // Fetch the processes.
                let mut process_info: Vec<restartmanager::RM_PROCESS_INFO> =
                    Vec::with_capacity(n_proc_info_needed as usize);
                n_proc_info = n_proc_info_needed;
                if restartmanager::RmGetList(
                    session_handle,
                    &mut n_proc_info_needed,
                    &mut n_proc_info,
                    process_info.as_mut_ptr(),
                    &mut reboot_reasons,
                ) != 0
                {
                    Err(format_err!("Failed to fetch list."))
                } else {
                    process_info.set_len(n_proc_info as usize);
                    let ents = process_info
                        .into_iter()
                        .map(|pi| Process {
                            process_id: pi.Process.dwProcessId,
                            start_time: (
                                pi.Process.ProcessStartTime.dwLowDateTime,
                                pi.Process.ProcessStartTime.dwHighDateTime,
                            ),
                            app_name: WideCString::from_ptr_str(&pi.strAppName[0])
                                .to_string_lossy(),
                            service_short_name: WideCString::from_ptr_str(
                                &pi.strServiceShortName[0],
                            ).to_string_lossy(),
                            application_type: pi.ApplicationType,
                            app_status: pi.AppStatus,
                        })
                        .collect();
                    Ok(ents)
                }
            }
        };
        let end_res = restartmanager::RmEndSession(session_handle);
        if end_res != 0 {
            panic!("Failed to end session: {}", end_res);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple() {
        let p = ::std::env::current_exe().unwrap();
        println!("Checking {:?}", p);
        let res = get_procs_using_path(p).unwrap();
        println!("{:#?}", res);
    }
}
