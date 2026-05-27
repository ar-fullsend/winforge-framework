use std::any::Any;
use std::ffi::{CStr, CString};
use std::path::Path;

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::error::{CoreError, CoreResult};

use super::manifest::PluginManifest;
use super::registry::{Plugin, PluginHost};

// ── ABI constants (mirrors winforge-plugin::abi) ────────────────────────────

const PLUGIN_CREATE_SYMBOL: &[u8] = b"wf_plugin_create\0";
const WF_OK: i32 = 0;

#[repr(C)]
struct WfVTable {
    plugin_data: *mut std::ffi::c_void,
    name: *const std::ffi::c_char,
    version: *const std::ffi::c_char,
    on_load: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char) -> i32,
    on_unload: unsafe extern "C" fn(*mut std::ffi::c_void) -> i32,
    destroy: unsafe extern "C" fn(*mut WfVTable),
}

unsafe impl Send for WfVTable {}
unsafe impl Sync for WfVTable {}

type WfPluginCreateFn = unsafe extern "C" fn() -> *mut WfVTable;

// ── Hash verification ────────────────────────────────────────────────────────

/// Verify the SHA-256 digest of `path` against `expected_hex` (case-insensitive).
pub fn verify_hash(path: &Path, expected_hex: &str) -> CoreResult<()> {
    let bytes = std::fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    let actual = hex::encode(digest);
    if actual != expected_hex.to_lowercase() {
        return Err(CoreError::Plugin(format!(
            "integrity check failed for '{}'\n  expected: {}\n  actual:   {}",
            path.display(),
            expected_hex,
            actual
        )));
    }
    Ok(())
}

// ── DynamicPlugin ────────────────────────────────────────────────────────────

/// Wraps a loaded shared library and its C vtable as a [`Plugin`].
///
/// The `Library` handle keeps the shared library mapped in memory for as long as
/// `DynamicPlugin` is alive. The vtable's `destroy` is called in `Drop`.
pub struct DynamicPlugin {
    vtable: *mut WfVTable,
    // The library must outlive the vtable.
    _lib: libloading::Library,
    plugin_name: String,
    plugin_version: String,
}

// SAFETY: WfVTable is Send (see above). Library is Send.
unsafe impl Send for DynamicPlugin {}
unsafe impl Sync for DynamicPlugin {}

impl DynamicPlugin {
    fn vtable(&self) -> &WfVTable {
        // SAFETY: vtable is valid for the lifetime of this struct.
        unsafe { &*self.vtable }
    }
}

impl Drop for DynamicPlugin {
    fn drop(&mut self) {
        // SAFETY: destroy is called exactly once here.
        unsafe { (self.vtable().destroy)(self.vtable) }
    }
}

#[async_trait]
impl Plugin for DynamicPlugin {
    fn name(&self) -> &str {
        &self.plugin_name
    }

    fn version(&self) -> &str {
        &self.plugin_version
    }

    async fn on_load(&mut self, host: &PluginHost) -> CoreResult<()> {
        // Serialize granted capabilities as a JSON array for the C ABI.
        let caps: Vec<String> = host.granted_capabilities.iter().map(|c| c.to_string()).collect();
        let caps_json = serde_json::to_string(&caps)
            .map_err(|e| CoreError::Plugin(e.to_string()))?;
        let caps_cstr = CString::new(caps_json)
            .map_err(|e| CoreError::Plugin(e.to_string()))?;

        let rc = unsafe {
            (self.vtable().on_load)(self.vtable().plugin_data, caps_cstr.as_ptr())
        };
        if rc != WF_OK {
            return Err(CoreError::Plugin(format!(
                "plugin '{}' on_load returned error code {rc}",
                self.plugin_name
            )));
        }
        Ok(())
    }

    async fn on_unload(&mut self) -> CoreResult<()> {
        let rc = unsafe { (self.vtable().on_unload)(self.vtable().plugin_data) };
        if rc != WF_OK {
            return Err(CoreError::Plugin(format!(
                "plugin '{}' on_unload returned error code {rc}",
                self.plugin_name
            )));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ── Loader ───────────────────────────────────────────────────────────────────

/// Load a plugin from its directory.
///
/// Steps:
/// 1. Parse `plugin.toml`
/// 2. Resolve the entry-point path relative to `plugin_dir`
/// 3. If `manifest.plugin.sha256` is set, verify the file hash
/// 4. `dlopen` / `LoadLibrary` the shared library
/// 5. Call `wf_plugin_create()` to obtain the vtable
/// 6. Return `(manifest, Box<dyn Plugin>)`
pub fn load_plugin(plugin_dir: &Path) -> CoreResult<(PluginManifest, Box<dyn Plugin>)> {
    let manifest = PluginManifest::load(plugin_dir)?;
    manifest.validate()?;

    let entry_path = plugin_dir.join(&manifest.plugin.entry_point);
    if !entry_path.exists() {
        return Err(CoreError::Plugin(format!(
            "entry_point '{}' not found for plugin '{}'",
            entry_path.display(),
            manifest.plugin.name,
        )));
    }

    if let Some(expected_hash) = &manifest.plugin.sha256 {
        info!(plugin = %manifest.plugin.name, "verifying plugin integrity");
        verify_hash(&entry_path, expected_hash)?;
    }

    info!(plugin = %manifest.plugin.name, path = %entry_path.display(), "loading plugin library");

    // SAFETY: loading an arbitrary shared library is inherently unsafe.
    // The host is responsible for only loading trusted, signed plugins.
    let lib = unsafe { libloading::Library::new(&entry_path) }.map_err(|e| {
        CoreError::Plugin(format!(
            "failed to load '{}': {e}",
            entry_path.display()
        ))
    })?;

    let create_fn: libloading::Symbol<WfPluginCreateFn> = unsafe {
        lib.get(PLUGIN_CREATE_SYMBOL)
    }
    .map_err(|e| {
        CoreError::Plugin(format!(
            "symbol 'wf_plugin_create' not found in '{}': {e}",
            entry_path.display()
        ))
    })?;

    let vtable: *mut WfVTable = unsafe { create_fn() };
    if vtable.is_null() {
        return Err(CoreError::Plugin(format!(
            "wf_plugin_create() returned null for plugin '{}'",
            manifest.plugin.name
        )));
    }

    // Read name/version from the vtable before handing it off.
    let plugin_name = unsafe {
        CStr::from_ptr((*vtable).name)
            .to_string_lossy()
            .into_owned()
    };
    let plugin_version = unsafe {
        CStr::from_ptr((*vtable).version)
            .to_string_lossy()
            .into_owned()
    };

    let plugin = DynamicPlugin {
        vtable,
        _lib: lib,
        plugin_name,
        plugin_version,
    };

    Ok((manifest, Box::new(plugin)))
}
