//! Reads dropped or chosen files via `gloo-file`.
//!
//! Ordering (PLAN.md section 5) is enforced here: multiple files dropped or
//! chosen at once are sorted lexicographically by name before their bytes
//! ever reach `unpack.rs`/`tx-core`, matching archive entry ordering.
//! Unpacking and analysis happen downstream in [`crate::hooks::use_queue`],
//! which owns the id/items state the results are merged into.

use gloo_file::futures::read_as_bytes;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileList as WebFileList;
use yew::prelude::*;

/// Returns a callback that takes a raw `web_sys::FileList` (from an
/// `<input type="file">` change event or a `DataTransfer`), reads every
/// file's bytes, sorts them lexicographically by name, and hands the
/// ordered `(name, bytes)` pairs to `on_loaded`.
#[hook]
pub fn use_file_load(on_loaded: Callback<Vec<(String, Vec<u8>)>>) -> Callback<WebFileList> {
    Callback::from(move |file_list: WebFileList| {
        let on_loaded = on_loaded.clone();
        let files: Vec<gloo_file::File> = gloo_file::FileList::from(file_list).to_vec();
        spawn_local(async move {
            let mut loaded = Vec::with_capacity(files.len());
            for file in &files {
                // An unreadable file (revoked permission, IO error mid-read) is
                // dropped rather than surfaced: there is nothing to queue for
                // bytes that never arrived.
                if let Ok(bytes) = read_as_bytes(file).await {
                    loaded.push((file.name(), bytes));
                }
            }
            loaded.sort_by(|a, b| a.0.cmp(&b.0));
            on_loaded.emit(loaded);
        });
    })
}
