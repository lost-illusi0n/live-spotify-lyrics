use std::ffi::OsString;
use std::mem;
use std::ops::Div;
use std::os::windows::ffi::OsStringExt;
use winapi::_core::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, HMODULE, LPCVOID, LPVOID};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::{
    EnumProcessModulesEx, GetModuleBaseNameA, GetModuleInformation, MODULEINFO,
};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::PROCESS_ALL_ACCESS;

pub struct SpotifyMemReader {}

const LIB_CEF: &'static str = "libcef.dll";
const SPOTIFY: &'static str = "Spotify.exe";

const PLAYING_ADDRESS: u32 = 0x016B9D3C;
const PLAYING_OFFSETS: [u32; 1] = [0x138];

#[macro_export]
macro_rules! c_void {
    ($arg:expr) => {
        &mut $arg as *mut _ as LPVOID
    };
}

impl SpotifyMemReader {
    unsafe fn read_pointer<T>(address: u32, offsets: &[u32], buffer: &mut T) -> bool {
        if !SpotifyMemReader::is_connected() {
            return false;
        };

        let mut address = address;
        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY).unwrap();
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        let spotify_base_address: u32 =
            SpotifyMemReader::get_module_base_address(SPOTIFY, pid).unwrap() as u32;

        ReadProcessMemory(
            handle,
            (spotify_base_address + address) as LPCVOID,
            c_void!(address),
            mem::size_of::<u32>(),
            null_mut(),
        );

        for i in 0..offsets.len() {
            let offset = offsets[i];

            let output: LPVOID = if i == (offsets.len() - 1) {
                c_void!(*buffer)
            } else {
                c_void!(address)
            };

            ReadProcessMemory(
                handle,
                (address + offset) as LPCVOID,
                output,
                mem::size_of_val(&output),
                null_mut(),
            );
        }

        CloseHandle(handle);

        return true;
    }

    pub unsafe fn is_connected() -> bool {
        SpotifyMemReader::find_pid_by_name(SPOTIFY).is_some()
    }

    pub unsafe fn is_playing() -> Option<bool> {
        if !SpotifyMemReader::is_connected() {
            return None;
        };

        let mut value: bool = false;

        return if SpotifyMemReader::read_pointer(PLAYING_ADDRESS, &PLAYING_OFFSETS, &mut value) {
            Some(value)
        } else {
            None
        };
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
                            // println!("{} | {}", process.th32ProcessID, value);
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

    unsafe fn get_module_base_address(module_name: &str, pid: u32) -> Option<u32> {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
        let mut h_mods: [HMODULE; 1024] = mem::MaybeUninit::zeroed().assume_init();
        let cb_needed: &mut DWORD = &mut 0;

        if EnumProcessModulesEx(
            handle,
            h_mods.as_mut_ptr(),
            mem::size_of_val(&h_mods) as u32,
            cb_needed,
            1,
        ) == 1
        {
            for i in 0..(cb_needed.div(mem::size_of::<HMODULE>() as u32)) {
                let h_mod: HMODULE = h_mods[i as usize];
                let mut info: MODULEINFO = mem::MaybeUninit::zeroed().assume_init();

                GetModuleInformation(
                    handle,
                    h_mod,
                    &mut info,
                    mem::size_of::<MODULEINFO>() as u32,
                );

                let mut name: Vec<u8> = vec![0; 256];

                GetModuleBaseNameA(handle, h_mod, name.as_mut_ptr() as *mut i8, 256);

                let parsed_name: String = String::from_utf8(Vec::from(&name[..256])).ok()?;

                if parsed_name.contains(module_name) {
                    return Some(info.lpBaseOfDll as u32);
                }
            }
        }

        return None;
    }
}
