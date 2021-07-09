use std::ffi::OsString;
use std::mem;
use std::ops::Div;
use std::os::windows::ffi::OsStringExt;

use winapi::_core::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, HMODULE, LPCVOID, LPVOID};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::{
    EnumProcessModulesEx, GetModuleBaseNameA, GetModuleInformation, MODULEINFO,
};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use winapi::um::winnt::PROCESS_ALL_ACCESS;

use crate::CurrentTrack;

pub struct SpotifyMemReader {}

const LIB_CEF: &'static str = "libcef.dll";
const SPOTIFY: &'static str = "Spotify.exe";

#[macro_export]
macro_rules! as_lpvoid {
    ($arg:expr) => {
        &mut $arg as *mut _ as LPVOID
    };
}

impl SpotifyMemReader {
    unsafe fn get_pointer_address_from_memory(address: usize, offsets: &[u32]) -> Option<usize> {
        if !SpotifyMemReader::is_connected() { return None; };

        let mut ptr_address: u32 = address.clone() as u32;
        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY).unwrap();
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        if ReadProcessMemory(
            handle,
            ptr_address as LPCVOID,
            as_lpvoid!(ptr_address),
            mem::size_of_val(&ptr_address),
            null_mut(),
        ) == 0 {
            println!("{}", GetLastError());
            panic!();
        };

        // println!("offset: {:x}", address);
        // println!("first: {:x}", ptr_address);

        for i in 0..offsets.len() {
            let offset = offsets[i];

            if i == (offsets.len() - 1) {
                ptr_address = ptr_address + offset
            } else {
                if ReadProcessMemory(
                    handle,
                    (ptr_address + offset) as LPCVOID,
                    as_lpvoid!(ptr_address),
                    mem::size_of_val(&ptr_address),
                    null_mut(),
                ) == 0 {
                    println!("{}", GetLastError());
                    panic!();
                };
            };
        }

        if CloseHandle(handle) == 0 {
            panic!()
        };

        return Some(ptr_address as usize);
    }

    unsafe fn read_pointer_from_memory<T>(address: usize, offsets: &[u32], buffer: &mut T) -> bool {
        if !SpotifyMemReader::is_connected() { return false; };

        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY).unwrap();
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        let final_address = match SpotifyMemReader::get_pointer_address_from_memory(address, offsets) {
            Some(address) => address,
            None => return false
        };

        if ReadProcessMemory(
            handle,
            final_address as LPCVOID,
            as_lpvoid!(*buffer),
            mem::size_of_val(&*buffer),
            null_mut(),
        ) == 0 {
            println!("{}", GetLastError());
            panic!();
        };

        if CloseHandle(handle) == 0 {
            panic!()
        };

        return true;
    }

    pub unsafe fn is_connected() -> bool {
        SpotifyMemReader::find_pid_by_name(SPOTIFY).is_some()
    }

    pub unsafe fn is_playing() -> Option<bool> {
        if !SpotifyMemReader::is_connected() {
            return None;
        };

        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY)?;
        let spotify_base_adr = SpotifyMemReader::get_module_base_address(SPOTIFY, pid)?;

        let mut value: bool = false;

        return if SpotifyMemReader::read_pointer_from_memory(spotify_base_adr + 0x016B9D3C, &[0x138], &mut value) {
            Some(value)
        } else {
            None
        };
    }

    pub unsafe fn current_track() -> Option<CurrentTrack> {
        if !SpotifyMemReader::is_playing().unwrap_or(false) {
            return None;
        }

        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY).unwrap();

        let lib_cef_base = SpotifyMemReader::get_module_base_address(LIB_CEF, pid).unwrap();
        let final_address = SpotifyMemReader::get_pointer_address_from_memory(lib_cef_base + 0x078C0F78, &[0x88, 0x2C, 0x20, 0x18, 0x0]).unwrap();

        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        let mut buffer: Vec<i8> = Vec::new();

        let mut state: CurrentTrackBufferState = CurrentTrackBufferState::Title;

        let mut title = mem::MaybeUninit::<String>::uninit();
        let mut author = mem::MaybeUninit::<String>::uninit();

        let mut i = 0;
        loop {
            let mut char: i8 = mem::zeroed();

            ReadProcessMemory(handle, (final_address + i) as LPCVOID, as_lpvoid!(char), mem::size_of::<i8>(), null_mut());
            i += 1;

            buffer.push(char);

            match state {
                CurrentTrackBufferState::Title => {
                    if buffer.iter().rev().take(3).collect::<Vec<&i8>>().eq(&vec![&-73, &-62, &32]) {
                        state = CurrentTrackBufferState::Author;
                        // remove garbage termination bytes
                        buffer.drain(buffer.len()-3..buffer.len());
                        title.as_mut_ptr().write(SpotifyMemReader::normalize_vec_i8_to_string(&buffer)?);
                        buffer.clear();
                    }
                }
                CurrentTrackBufferState::Author => {
                    if buffer.last().unwrap().eq(&0) {
                        // remove garbage 0 byte
                        buffer.remove(buffer.len() - 1);
                        author.as_mut_ptr().write(SpotifyMemReader::normalize_vec_i8_to_string(&buffer)?);
                        break
                    }
                }
            }
        }

        let title = title.assume_init();
        let author = author.assume_init();

        return Some(CurrentTrack { title, author })
    }

    fn normalize_vec_i8_to_string(chars: &Vec<i8>) -> Option<String> {
        Some(String::from_utf8(chars.iter().map(|u| u.clone() as u8).collect::<Vec<u8>>()).ok()?.trim_matches(0 as char).trim().to_string()) // wtf? make this look normal
    }

    unsafe fn find_pid_by_name(name: &str) -> Option<u32> {
        let mut process: PROCESSENTRY32W = mem::MaybeUninit::zeroed().assume_init();
        process.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        if Process32FirstW(snapshot, &mut process) == 1 {
            while Process32NextW(snapshot, &mut process) != 0 {
                let process_name: OsString = OsString::from_wide(&process.szExeFile);

                match process_name.into_string() {
                    Ok(value) => {
                        if value.contains(name) {
                            CloseHandle(snapshot);
                            return Some(process.th32ProcessID);
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        CloseHandle(snapshot);
        return None;
    }

    unsafe fn get_module_base_address(module_name: &str, pid: u32) -> Option<usize> {
        let proc = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        let mut h_mods: [HMODULE; 1024] = mem::MaybeUninit::zeroed().assume_init();
        let cb_needed: &mut DWORD = &mut 0;

        if EnumProcessModulesEx(
            proc,
            h_mods.as_mut_ptr(),
            mem::size_of_val(&h_mods) as u32,
            cb_needed,
            1,
        ) == 1 {
            for i in 0..cb_needed.div(mem::size_of::<HMODULE>() as u32) {
                let module_h: HMODULE = h_mods[i as usize];
                let mut info: MODULEINFO = mem::MaybeUninit::zeroed().assume_init();

                GetModuleInformation(
                    proc,
                    module_h,
                    &mut info,
                    mem::size_of::<MODULEINFO>() as u32,
                );

                let mut name: Vec<u8> = vec![0; 256];

                GetModuleBaseNameA(proc, module_h, name.as_mut_ptr() as *mut i8, 256);

                let parsed_name: String = String::from_utf8(Vec::from(&name[..256])).ok()?;

                if parsed_name.contains(module_name) {
                    return Some(info.lpBaseOfDll as usize) ;
                }
            }
        }

        CloseHandle(proc);

        return None;
    }
}

enum CurrentTrackBufferState {
    Title,
    Author
}