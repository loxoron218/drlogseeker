use std::fs::read;
use std::cmp::Ordering::{Greater, Less};
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc::channel, Mutex};
use std::thread::{self, spawn};

use gio::ListStore;
use glib::{clone, idle_add_local, MainContext};
use glib::ControlFlow::{Break, Continue};
use gtk4::{Button, ProgressBar, StringObject};
use libadwaita::prelude::WidgetExt;

use crate::data::app_state::{AppState, DRResult};
use crate::file_manager::file_ops::find_log_files;
use crate::utils::constants::get_dr_regex;

/// Scans a directory for log files asynchronously, updates the UI with progress,
/// and populates the list store with the results.
///
/// This function spawns a primary worker thread to find files and then distributes
/// the analysis of each file across a pool of secondary worker threads (sized to the number of CPU cores).
/// Communication with the GTK UI thread is handled via MPSC channels and `glib::idle_add_local`.
pub fn scan_directory(path: PathBuf, app_state: Arc<Mutex<AppState>>, progress_bar: ProgressBar, list_store:ListStore, scan_button: Button, clear_button: Button) {
    let (progress_tx, progress_rx_inner) = channel::<(usize, usize)>();
    let progress_rx = Arc::new(Mutex::new(progress_rx_inner));
    let (results_tx, results_rx_inner) = channel::<Vec<DRResult>>();
    let results_rx = Arc::new(Mutex::new(results_rx_inner));
    spawn(move || {
        let mut files = Vec::new();
        if find_log_files(&path, &mut files).is_err() {
            results_tx.send(Vec::new()).ok();
            return;
        }
        let total_files = files.len();
        if total_files == 0 {
            progress_tx.send((0, 0)).ok();
            results_tx.send(Vec::new()).expect("Failed to send empty results");
            return;
        }
        let (file_tx, file_rx) = channel::<PathBuf>();
        let file_rx = Arc::new(Mutex::new(file_rx));
        let (result_tx_worker, result_rx_worker) = channel::<DRResult>();
        let num_cpus = thread::available_parallelism().map(|p| p.get()).unwrap_or(1);
        let num_workers = num_cpus.max(1);
        let mut handles = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            let file_rx_clone = Arc::clone(&file_rx);
            let result_tx_worker_clone = result_tx_worker.clone();
            handles.push(spawn(move || {
                loop {
                    let file_path_result = {
                        let rx = file_rx_clone.lock().unwrap();
                        rx.recv()
                    };
                    match file_path_result {
                        Ok(file_path) => {
                            let result = analyze_file(&file_path);
                            if result_tx_worker_clone.send(result).is_err() {
                                break;
                            }
                        },
                        Err(_) => break,
                    }
                }
            }));
        }
        for file_path in files {
            file_tx.send(file_path).ok();
        }
        drop(file_tx);
        let mut collected_results = Vec::with_capacity(total_files);
        for i in 0..total_files {
            if let Ok(result) = result_rx_worker.recv() {
                collected_results.push(result);
                progress_tx.send((i + 1, total_files)).ok();
            } else {
                eprintln!("Worker result channel closed unexpectedly.");
                break;
            }
        }
        for handle in handles {
            handle.join().expect("Worker thread panicked");
        }
        drop(result_tx_worker);
        let results = collected_results;
        results_tx.send(results).ok();
    });
    idle_add_local(clone!(@strong progress_bar, @strong progress_rx => move || {
        if let Ok((current, total)) = progress_rx.lock().unwrap().try_recv() {
            if total > 0 {
                progress_bar.set_fraction(current as f64 / total as f64);
            }
            Continue
        } else {
            Break
        }
    }));
    let results_rx_clone = results_rx.clone();
    idle_add_local(clone!(@strong list_store, @strong app_state, @strong progress_bar, @strong scan_button, @strong clear_button => move || {
        if let Ok(results) = results_rx_clone.lock().unwrap().try_recv() {
            if let Ok(mut state) = app_state.lock() {
                state.results = results;
                update_ui(&list_store, &state.results);
            }
            progress_bar.set_visible(false);
            scan_button.set_sensitive(true);
            clear_button.set_sensitive(true);
            Break
        } else {
            Continue
        }
    }));
}

/// Analyzes a single log file to extract its DR (Dynamic Range) value.
///
/// It reads the file content and uses a regular expression to find the DR value.
/// If the file cannot be read or the value cannot be parsed, it returns a `DRResult`
/// indicating an error.
pub fn analyze_file(path: &Path) -> DRResult {
    let content = match read(path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => return create_error_result(path),
    };
    let dr_value = get_dr_regex()
        .captures(&content)
        .and_then(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map(|m| m.as_str())
                .and_then(|val| {
                    if val == "ERR" {
                        None
                    } else {
                        val.parse::<u8>().ok()
                    }
                })
        });
    DRResult {
        filename: path.file_name().unwrap().to_string_lossy().into_owned(),
        path: path.to_path_buf(),
        dr_value,
        scanned: true,
    }
}

/// Creates a `DRResult` that represents a scan error for a given file path.
///
/// The `scanned` flag is set to `true` to distinguish it from a pending file.
pub fn create_error_result(path: &Path) -> DRResult {
    DRResult {
        filename: path.file_name().unwrap().to_string_lossy().into_owned(),
        path: path.to_path_buf(),
        dr_value: None,
        scanned: true,
    }
}

/// Clears and repopulates the UI's list store with sorted scan results.
///
/// This function ensures that the UI update occurs on the main GTK thread.
/// The results are sorted with the following priority:
/// 1. Descending DR value (highest first).
/// 2. Alphabetical by path for files with the same DR value.
/// 3. Files with errors (`ERR`) are grouped after successfully scanned files.
/// 4. Unscanned files (`PENDING`) are shown last.
pub fn update_ui(list_store: &ListStore, results: &[DRResult]) {
    let results = results.to_vec();
    let list_store = list_store.clone();
    MainContext::default().invoke_local(move || {
        list_store.remove_all();
        let mut sorted_results = results;
        sorted_results.sort_by(|a, b| {
            match (a.dr_value, b.dr_value) {
                (Some(a_val), Some(b_val)) => b_val.cmp(&a_val)
                    .then_with(|| a.path.cmp(&b.path)),
                (Some(_), None) => Less,
                (None, Some(_)) => Greater,
                (None, None) => match (a.scanned, b.scanned) {
                    (true, true) | (false, false) => a.path.cmp(&b.path),
                    (true, false) => Greater,
                    (false, true) => Less,
                }
            }
        });
        for result in sorted_results {
            let dr_text = match (result.dr_value, result.scanned) {
                (Some(dr), _) => dr.to_string(),
                (None, true) => "ERR".to_string(),
                (None, false) => "PENDING".to_string(),
            };
            let text = format!(
                "{}	{}	{}",
                result.filename,
                result.path.to_string_lossy(),
                dr_text
            );
            list_store.append(&StringObject::new(&text));
        }
    });
}