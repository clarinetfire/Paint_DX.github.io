//! Autosave and Crash Recovery Manager for Paint DX.
//!
//! Periodically saves working state to an autosave file in the user's local app data
//! and provides crash recovery detection upon application restart.

use paint_core::frames::AnimationDocument;
use paint_core::palette::ColorRgba;
use paint_core::project::{ProjectData, ProjectError};

#[cfg(not(target_arch = "wasm32"))]
use paint_core::project::{load_project_from_path, save_project_to_path};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
pub struct AutosaveManager {
    recovery_dir: PathBuf,
    last_autosave: Instant,
    autosave_interval: Duration,
    has_unsaved_changes: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for AutosaveManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AutosaveManager {
    pub fn new() -> Self {
        let recovery_dir = Self::determine_recovery_dir();
        let _ = fs::create_dir_all(&recovery_dir);

        Self {
            recovery_dir,
            last_autosave: Instant::now(),
            autosave_interval: Duration::from_secs(30),
            has_unsaved_changes: false,
        }
    }

    fn determine_recovery_dir() -> PathBuf {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            PathBuf::from(local_app_data)
                .join("PaintDX")
                .join("recovery")
        } else {
            std::env::temp_dir().join("PaintDX").join("recovery")
        }
    }

    pub fn recovery_file_path(&self) -> PathBuf {
        self.recovery_dir.join("autosave.paintproj")
    }

    pub fn session_lock_path(&self) -> PathBuf {
        self.recovery_dir.join("session.lock")
    }

    /// Mark that document has been modified since last save.
    pub fn mark_dirty(&mut self) {
        self.has_unsaved_changes = true;
    }

    /// Mark that document is clean (saved to disk).
    pub fn mark_clean(&mut self) {
        self.has_unsaved_changes = false;
        self.clear_recovery_files();
    }

    /// Check if a recovery file from a previous unclosed session exists.
    pub fn has_recoverable_session(&self) -> bool {
        self.recovery_file_path().exists()
    }

    /// Load the recoverable project from disk.
    pub fn load_recovery_project(&self) -> Result<ProjectData, ProjectError> {
        let path = self.recovery_file_path();
        load_project_from_path(path)
    }

    /// Perform periodic autosave if dirty and interval elapsed.
    pub fn tick(
        &mut self,
        doc: &AnimationDocument,
        palette: &[ColorRgba; 28],
        current_path: Option<&Path>,
    ) -> bool {
        if !self.has_unsaved_changes {
            return false;
        }

        if self.last_autosave.elapsed() >= self.autosave_interval
            && self.perform_autosave(doc, palette, current_path).is_ok()
        {
            self.last_autosave = Instant::now();
            return true;
        }
        false
    }

    /// Force immediate autosave.
    pub fn perform_autosave(
        &self,
        doc: &AnimationDocument,
        palette: &[ColorRgba; 28],
        current_path: Option<&Path>,
    ) -> Result<(), String> {
        let _ = fs::create_dir_all(&self.recovery_dir);
        let project = ProjectData::from_doc(doc.clone(), Some(*palette));

        let temp_path = self.recovery_dir.join("autosave.tmp");
        let final_path = self.recovery_file_path();

        save_project_to_path(&project, &temp_path).map_err(|e| format!("Save error: {}", e))?;

        // Atomic rename (or replace on Windows)
        if final_path.exists() {
            let _ = fs::remove_file(&final_path);
        }
        fs::rename(&temp_path, &final_path).map_err(|e| format!("Rename error: {}", e))?;

        // Update lock file with timestamp and original file path if any
        let lock_path = self.session_lock_path();
        let path_str = current_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        let _ = fs::write(
            lock_path,
            format!("Path: {}\nTimestamp: {:?}\n", path_str, Instant::now()),
        );

        Ok(())
    }

    /// Remove autosave files on clean exit or manual save.
    pub fn clear_recovery_files(&self) {
        let _ = fs::remove_file(self.recovery_file_path());
        let _ = fs::remove_file(self.session_lock_path());
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for AutosaveManager {
    fn drop(&mut self) {
        // If not dirty on drop, remove recovery files
        if !self.has_unsaved_changes {
            self.clear_recovery_files();
        }
    }
}

// ---------------------------------------------------------------------------
// Web (wasm32) in-memory stub implementation
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub struct AutosaveManager {
    has_unsaved_changes: bool,
}

#[cfg(target_arch = "wasm32")]
impl Default for AutosaveManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
impl AutosaveManager {
    pub fn new() -> Self {
        Self {
            has_unsaved_changes: false,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.has_unsaved_changes = true;
    }

    pub fn mark_clean(&mut self) {
        self.has_unsaved_changes = false;
    }

    pub fn has_recoverable_session(&self) -> bool {
        false
    }

    pub fn load_recovery_project(&self) -> Result<ProjectData, ProjectError> {
        Err(ProjectError::MissingManifest)
    }

    pub fn tick(
        &mut self,
        _doc: &AnimationDocument,
        _palette: &[ColorRgba; 28],
        _current_path: Option<&std::path::Path>,
    ) -> bool {
        false
    }

    pub fn perform_autosave(
        &self,
        _doc: &AnimationDocument,
        _palette: &[ColorRgba; 28],
        _current_path: Option<&std::path::Path>,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn clear_recovery_files(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint_core::palette::{WHITE, XP_PALETTE_28};

    #[test]
    fn test_autosave_lifecycle() {
        let temp_dir = std::env::temp_dir().join("paint_dx_autosave_test");
        let _ = fs::remove_dir_all(&temp_dir);
        let _ = fs::create_dir_all(&temp_dir);

        let mut manager = AutosaveManager::new();
        manager.recovery_dir = temp_dir.clone();

        assert!(!manager.has_recoverable_session());

        let doc = AnimationDocument::new(64, 64, WHITE);
        let palette = XP_PALETTE_28;

        manager.mark_dirty();
        let saved = manager.perform_autosave(&doc, &palette, None);
        assert!(saved.is_ok());

        assert!(manager.has_recoverable_session());

        let recovered = manager.load_recovery_project();
        assert!(recovered.is_ok());
        let proj = recovered.unwrap();
        assert_eq!(proj.width, 64);
        assert_eq!(proj.height, 64);

        manager.mark_clean();
        assert!(!manager.has_recoverable_session());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
