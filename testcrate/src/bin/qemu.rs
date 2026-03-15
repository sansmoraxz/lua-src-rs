use core::ffi::{c_int, c_void};
use std::{thread, time::Duration};

use testcrate::{
    luaL_error, luaL_loadstring, luaL_newstate, luaL_openlibs, lua_getglobal, lua_pcall,
    lua_pushcclosure, to_string,
};

const UCID_CHUNK: &[u8] = b"\xF0\x9F\x98\x80 = '\xF0\x9F\x8C\x9A\xEF\xB8\x8E'\0";
#[cfg(feature = "lua54")]
const UCID_NAME: &[u8] = b"\xF0\x9F\x98\x80\0";
#[cfg(feature = "lua54")]
const UCID_VALUE: &str = "\u{1F31A}\u{FE0E}";

#[cfg(feature = "lua51")]
fn selected_lua() -> &'static str {
    "lua51"
}

#[cfg(feature = "lua52")]
fn selected_lua() -> &'static str {
    "lua52"
}

#[cfg(feature = "lua53")]
fn selected_lua() -> &'static str {
    "lua53"
}

#[cfg(feature = "lua54")]
fn selected_lua() -> &'static str {
    "lua54"
}

#[cfg(feature = "lua51")]
fn expected_version() -> &'static str {
    "Lua 5.1"
}

#[cfg(feature = "lua52")]
fn expected_version() -> &'static str {
    "Lua 5.2"
}

#[cfg(feature = "lua53")]
fn expected_version() -> &'static str {
    "Lua 5.3"
}

#[cfg(feature = "lua54")]
fn expected_version() -> &'static str {
    "Lua 5.4"
}

fn main() {
    esp_idf_sys::link_patches();

    println!("LUA_SRC_QEMU_START lua={}", selected_lua());

    match run() {
        Ok(()) => println!("LUA_SRC_QEMU_OK lua={}", selected_lua()),
        Err(err) => {
            println!("LUA_SRC_QEMU_FAIL lua={} error={err}", selected_lua());
            panic!("{err}");
        }
    }

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn run() -> Result<(), &'static str> {
    unsafe {
        let state = luaL_newstate();
        if state.is_null() {
            return Err("luaL_newstate returned null");
        }

        luaL_openlibs(state);

        lua_getglobal(state, c"_VERSION".as_ptr());
        if to_string(state, -1) != expected_version() {
            return Err("unexpected Lua version");
        }

        if luaL_loadstring(state, c"return 'ok'".as_ptr()) != 0 {
            return Err("luaL_loadstring failed");
        }
        if lua_pcall(state, 0, 1, 0) != 0 {
            return Err("lua_pcall failed");
        }
        if to_string(state, -1) != "ok" {
            return Err("unexpected Lua result");
        }

        unsafe extern "C-unwind" fn it_panics(state: *mut c_void) -> c_int {
            unsafe { luaL_error(state, c"exception!".as_ptr()) }
        }

        lua_pushcclosure(state, it_panics, 0);
        if lua_pcall(state, 0, 0, 0) != 2 {
            return Err("lua_pcall did not return LUA_ERRRUN");
        }
        if to_string(state, -1) != "exception!" {
            return Err("unexpected exception text");
        }

        let ret = luaL_loadstring(state, UCID_CHUNK.as_ptr().cast());
        #[cfg(feature = "lua54")]
        {
            if ret != 0 {
                return Err("unicode identifier chunk did not compile");
            }
            if lua_pcall(state, 0, 0, 0) != 0 {
                return Err("unicode identifier chunk did not run");
            }
            lua_getglobal(state, UCID_NAME.as_ptr().cast());
            if to_string(state, -1) != UCID_VALUE {
                return Err("unexpected unicode identifier value");
            }
        }

        #[cfg(not(feature = "lua54"))]
        {
            if ret == 0 {
                return Err("unicode identifier chunk unexpectedly compiled");
            }
        }
    }

    Ok(())
}
