use std::ffi::{c_char, c_int, c_void};

/// Return code from C ABI plugin functions.
pub type WfResult = c_int;
pub const WF_OK: WfResult = 0;
pub const WF_ERR_GENERAL: WfResult = 1;
pub const WF_ERR_CAPABILITY_DENIED: WfResult = 2;

/// Opaque pointer to the plugin's internal state.
pub type PluginData = *mut c_void;

/// The stable ABI table that a plugin DLL exposes to the host.
///
/// All pointers must remain valid from `wf_plugin_create()` until `destroy()`.
/// The host owns this struct after `wf_plugin_create()` returns and calls
/// `destroy` exactly once when unloading.
#[repr(C)]
pub struct WfVTable {
    /// Opaque plugin instance data; passed verbatim to every function pointer.
    pub plugin_data: PluginData,

    /// Null-terminated UTF-8 plugin name. Must be valid for the lifetime of the vtable.
    pub name: *const c_char,

    /// Null-terminated UTF-8 plugin version.
    pub version: *const c_char,

    /// Called once after the library is loaded.
    /// `capabilities_json` is a null-terminated JSON array of granted capability strings,
    /// e.g. `["events:publish","filesystem:read"]`.
    pub on_load: unsafe extern "C" fn(data: PluginData, capabilities_json: *const c_char) -> WfResult,

    /// Called once before the library is unloaded.
    pub on_unload: unsafe extern "C" fn(data: PluginData) -> WfResult,

    /// Drops both `plugin_data` and this vtable struct.
    /// The host calls this after `on_unload` returns.
    pub destroy: unsafe extern "C" fn(vtable: *mut WfVTable),
}

// SAFETY: the raw pointers inside WfVTable are owned by the plugin and valid
// for the plugin's lifetime. The host always calls the vtable functions
// single-threaded (protected by PluginRegistry's sequential HashMap access).
unsafe impl Send for WfVTable {}
unsafe impl Sync for WfVTable {}

/// Well-known symbol name that every WinForge plugin DLL must export.
pub const PLUGIN_CREATE_SYMBOL: &[u8] = b"wf_plugin_create\0";

/// Type signature of the exported factory function.
///
/// # Safety
/// The returned pointer must point to a heap-allocated `WfVTable` whose
/// `destroy` function will free both the vtable and `plugin_data`.
pub type WfPluginCreateFn = unsafe extern "C" fn() -> *mut WfVTable;

/// Declare the plugin entry point and wire up the C ABI automatically.
///
/// # Example
/// ```rust,ignore
/// use winforge_plugin::prelude::*;
///
/// struct MyPlugin;
///
/// #[async_trait]
/// impl Plugin for MyPlugin { /* ... */ }
///
/// winforge_plugin::export_plugin!(MyPlugin::new());
/// ```
#[macro_export]
macro_rules! export_plugin {
    ($constructor:expr) => {
        mod __wf_plugin_export {
            use super::*;
            use std::ffi::{CString, c_char, c_void};
            use $crate::abi::{WfResult, WfVTable, PluginData, WF_OK, WF_ERR_GENERAL};

            struct State {
                plugin: Box<dyn $crate::Plugin>,
                name_c: CString,
                version_c: CString,
            }

            unsafe extern "C" fn on_load(data: PluginData, caps_json: *const c_char) -> WfResult {
                let state = unsafe { &mut *(data as *mut State) };
                let caps_str = unsafe {
                    std::ffi::CStr::from_ptr(caps_json).to_str().unwrap_or("[]")
                };
                let caps: Vec<String> = serde_json::from_str(caps_str).unwrap_or_default();
                let granted: std::collections::HashSet<$crate::Capability> = caps
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                let host = $crate::PluginHost { granted_capabilities: granted };

                // Block on the async call using a new single-threaded runtime scoped here.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("tokio rt");
                match rt.block_on(state.plugin.on_load(&host)) {
                    Ok(()) => WF_OK,
                    Err(_) => WF_ERR_GENERAL,
                }
            }

            unsafe extern "C" fn on_unload(data: PluginData) -> WfResult {
                let state = unsafe { &mut *(data as *mut State) };
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("tokio rt");
                match rt.block_on(state.plugin.on_unload()) {
                    Ok(()) => WF_OK,
                    Err(_) => WF_ERR_GENERAL,
                }
            }

            unsafe extern "C" fn destroy(vtable: *mut WfVTable) {
                let vtable = unsafe { Box::from_raw(vtable) };
                // Drop State (and inner plugin) by reconstructing the Box.
                let _ = unsafe { Box::from_raw(vtable.plugin_data as *mut State) };
                // vtable itself is dropped here.
            }

            #[no_mangle]
            pub extern "C" fn wf_plugin_create() -> *mut WfVTable {
                let plugin: Box<dyn $crate::Plugin> = Box::new($constructor);
                let name_c = CString::new(plugin.name()).unwrap_or_default();
                let version_c = CString::new(plugin.version()).unwrap_or_default();

                let state = Box::new(State { plugin, name_c, version_c });
                let name_ptr = state.name_c.as_ptr();
                let version_ptr = state.version_c.as_ptr();
                let data = Box::into_raw(state) as PluginData;

                Box::into_raw(Box::new(WfVTable {
                    plugin_data: data,
                    name: name_ptr,
                    version: version_ptr,
                    on_load,
                    on_unload,
                    destroy,
                }))
            }
        }
    };
}
