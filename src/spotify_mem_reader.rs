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

// macro to make this ugly cast into a c type void readable
#[macro_export]
macro_rules! as_lpvoid {
    ($arg:expr) => {
        &mut $arg as *mut _ as LPVOID
    };
}

impl SpotifyMemReader {
    unsafe fn get_pointer_address_from_memory(address: usize, offsets: &[u32]) -> Option<usize> {
        // check if the spotify process is open
        if !SpotifyMemReader::is_connected() { return None; };

        // this ptr_address will mutate to new pointer addresses based on [offsets]
        let mut ptr_address: u32 = address.clone() as u32;

        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY).unwrap();
        // get a handle on the spotify process
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        // read the first pointer offset address
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

        // println!("ptr: {:x}", address);
        // println!("first: {:x}", ptr_address);

        // iterate through our offsets
        for i in 0..offsets.len() {
            let offset = offsets[i];

            // if this is the last offset, then the value at this offset is the actual value and not another pointer
            // so we don't want to read this address
            if i == (offsets.len() - 1) {
                ptr_address = ptr_address + offset
            } else {
                // read the new pointer and write it to ptr_address to be used again
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

        // println!("final {:x}", ptr_address);

        return Some(ptr_address as usize);
    }

    unsafe fn read_pointer_from_memory<T>(address: usize, offsets: &[u32], buffer: &mut T) -> bool {
        // check if the spotify process is open
        if !SpotifyMemReader::is_connected() { return false; };

        let pid = SpotifyMemReader::find_pid_by_name(SPOTIFY).unwrap();
        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        // iterate through our offsets to get the actual pointer for our value
        let final_address = match SpotifyMemReader::get_pointer_address_from_memory(address, offsets) {
            Some(address) => address,
            None => return false
        };

        // read the value from memory and write it into bfufer
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
        // get the base address for the Spotify.exe module
        let spotify_base_adr = SpotifyMemReader::get_module_base_address(SPOTIFY, pid)?;

        let mut value: bool = false;

        // is_playing is at spotify_base_adr + 0x016C9300 with offsets [0x34, 0x0, 0x30, 0x4, 0x48], then write the byte into our bool value.
        return if SpotifyMemReader::read_pointer_from_memory(spotify_base_adr + 0x016C9300, &[0x34, 0x0, 0x30, 0x4, 0x48], &mut value) {
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

        // the current_track pointer we are using is based off lib_cef
        let lib_cef_base = SpotifyMemReader::get_module_base_address(LIB_CEF, pid).unwrap();
        // the address we want is lib_cef + 0x07940354, with the offsets x38, 0x3C, 0x4, 0x18, 0x0
        let final_address = SpotifyMemReader::get_pointer_address_from_memory(lib_cef_base + 0x07940354, &[0x38, 0x3C, 0x4, 0x18, 0x0]).unwrap();

        let handle = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        // keep a buffer of i8 for each char we read
        let mut buffer: Vec<i8> = Vec::new();

        // store our state of what we are reading in memory, as the title and author both have different ending sequences.
        let mut state: CurrentTrackBufferState = CurrentTrackBufferState::Title;

        let mut title = mem::MaybeUninit::<String>::uninit();
        let mut author = mem::MaybeUninit::<String>::uninit();

        // store the index of what byte we are on.
        let mut i = 0;
        loop {
            // the char for this iteration
            let mut char: i8 = mem::zeroed();
            // read the char from memory into the char var
            ReadProcessMemory(handle, (final_address + i) as LPCVOID, as_lpvoid!(char), mem::size_of::<i8>(), null_mut());
            i += 1;

            // add the char into our buffer
            buffer.push(char);

            // match our state to know what we are reading
            match state {
                CurrentTrackBufferState::Title => {
                    // the title end with the sequence of bytes -73, -62, 32
                    if buffer.iter().rev().take(3).collect::<Vec<&i8>>().eq(&vec![&-73, &-62, &32]) {
                        // we finished reading the title, start reading the author
                        state = CurrentTrackBufferState::Author;
                        // remove garbage termination bytes
                        buffer.drain(buffer.len()-3..buffer.len());
                        // write the buffer into title
                        title.as_mut_ptr().write(SpotifyMemReader::normalize_vec_i8_to_string(&buffer)?);
                        // clear our buffer so we can read title
                        buffer.clear();
                    }
                }
                CurrentTrackBufferState::Author => {
                    // once we read the byte 0, we finish reading.
                    if buffer.last().unwrap().eq(&0) {
                        // remove the garbage 0 byte
                        buffer.remove(buffer.len() - 1);
                        // write the buffer into author
                        author.as_mut_ptr().write(SpotifyMemReader::normalize_vec_i8_to_string(&buffer)?);
                        break
                    }
                }
            }
        }

        // tell rust that the values were written
        let title = title.assume_init();
        let author = author.assume_init();

        return Some(CurrentTrack { title, author })
    }

    fn normalize_vec_i8_to_string(chars: &Vec<i8>) -> Option<String> {
        Some(
            String::from_utf8(
                chars.iter()
                    .map(|u| u.clone() as u8) // make our i8s into u8s
                    .collect::<Vec<u8>>()           // collect them into a vec
            ).ok()?                                 // create a string from our vec
                .trim_matches(0 as char)      // remove extra garbage at the end
                .trim()                            // trim any whitespaces
                .to_string()                       // convert it back to a String
        )
    }

    unsafe fn find_pid_by_name(name: &str) -> Option<u32> {
        // allocate our PROCESSENTRY32W
        let mut process: PROCESSENTRY32W = mem::zeroed();
        process.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;

        // create a snapshot of processes
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        // iterate through the processes in our snapshot and write the process into our process variable
        if Process32FirstW(snapshot, &mut process) == 1 {
            while Process32NextW(snapshot, &mut process) != 0 {
                // read the process name
                let process_name: OsString = OsString::from_wide(&process.szExeFile);

                match process_name.into_string() {
                    Ok(value) => {
                        // the name we read has alot of extra garbage at the end so just checking if it contains our wanted value should be fine
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
        // get a handle for the process given
        let proc = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);

        // allocate our modules
        let mut h_mods: [HMODULE; 1024] = mem::MaybeUninit::zeroed().assume_init();
        let cb_needed: &mut DWORD = &mut 0;

        // tell win32 to write the modules for this process into h_mods
        if EnumProcessModulesEx(
            proc,
            h_mods.as_mut_ptr(),
            mem::size_of_val(&h_mods) as u32,
            cb_needed,
            1,
        ) == 1 {
            // iterate through the modules based on the size of a single module
            for i in 0..cb_needed.div(mem::size_of::<HMODULE>() as u32) {
                // access the module
                let module_h: HMODULE = h_mods[i as usize];

                // allocate moduleinfo
                let mut info: MODULEINFO = mem::MaybeUninit::zeroed().assume_init();

                // write module info into moduleinfo
                GetModuleInformation(
                    proc,
                    module_h,
                    &mut info,
                    mem::size_of::<MODULEINFO>() as u32,
                );

                // here we read the name. to do so we allocate a name buffer
                let mut name: Vec<u8> = vec![0; 256];

                // write the name into name buffer
                GetModuleBaseNameA(proc, module_h, name.as_mut_ptr() as *mut i8, 256);

                // parse the name into a string
                let parsed_name: String = String::from_utf8(Vec::from(&name[..256])).ok()?;

                // name contains garbage so checking if it contains our wanted module name is good enough
                if parsed_name.contains(module_name) {
                    // lpBaseOfDll is the base address
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