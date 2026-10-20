//! Project serialization and `.paintproj` archive format.
//!
//! A `.paintproj` file is a ZIP archive containing:
//! - `manifest.json`: JSON metadata holding canvas dimensions, layers metadata, custom palette, animation frames, and tags.
//! - `layers/layer_{id}.png`: Lossless RGBA PNG image buffer for backward compatibility.
//! - `cels/frame_{f}_layer_{id}.png`: Lossless RGBA PNG image buffer for each animation cel.

use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::frames::{AnimTag, AnimationDocument, Cel, Frame, LayerMetadata};
use crate::image::PixelBuffer;
use crate::io::{load_from_memory, save_to_memory, ImageFileFormat, ImageIoError};
use crate::layers::{BlendMode, LayerId, LayerStack};
use crate::palette::ColorRgba;

pub const PROJECT_VERSION: u32 = 2;

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Image codec error: {0}")]
    Image(#[from] ImageIoError),
    #[error("Missing project manifest in archive")]
    MissingManifest,
    #[error("Layer or Cel file not found in archive: {0}")]
    MissingLayerFile(String),
    #[error("Invalid project data: {0}")]
    InvalidData(String),
}

/// Metadata entry for a single layer in the project manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerManifestEntry {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: u8,
    pub blend: BlendMode,
    pub offset_x: i32,
    pub offset_y: i32,
    pub image_path: String,
}

/// Metadata entry for a single cel in a frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CelManifestEntry {
    pub layer_id: u64,
    pub offset_x: i32,
    pub offset_y: i32,
    pub image_path: String,
}

/// Metadata entry for an animation frame in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameManifestEntry {
    pub duration_ms: u32,
    pub cels: Vec<CelManifestEntry>,
}

/// Animation tag metadata in the project manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimTagManifestEntry {
    pub name: String,
    pub from_frame: usize,
    pub to_frame: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<[u8; 3]>,
}

/// Root JSON manifest structure for `.paintproj`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<Vec<ColorRgba>>,
    pub active_layer_id: u64,
    #[serde(default)]
    pub active_frame_index: usize,
    pub layers: Vec<LayerManifestEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<FrameManifestEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<AnimTagManifestEntry>,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            version: PROJECT_VERSION,
            width: 800,
            height: 600,
            fps: 12,
            palette: None,
            active_layer_id: 1,
            active_frame_index: 0,
            layers: Vec::new(),
            frames: Vec::new(),
            tags: Vec::new(),
        }
    }
}

/// In-memory representation of a full Paint DX project with layers and animation frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectData {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub palette: Option<[ColorRgba; 28]>,
    pub layer_stack: LayerStack,
    pub doc: AnimationDocument,
    pub tags: Vec<AnimTag>,
}

impl ProjectData {
    /// Creates a new ProjectData from a LayerStack and an optional 28-color palette.
    pub fn from_layer_stack(layer_stack: LayerStack, palette: Option<[ColorRgba; 28]>) -> Self {
        let width = layer_stack.width;
        let height = layer_stack.height;
        let doc = AnimationDocument::from_layer_stack(layer_stack.clone(), 12);
        let tags = doc.tags.clone();
        Self {
            width,
            height,
            fps: 12,
            palette,
            layer_stack,
            doc,
            tags,
        }
    }

    /// Creates a ProjectData from an AnimationDocument.
    pub fn from_doc(doc: AnimationDocument, palette: Option<[ColorRgba; 28]>) -> Self {
        let width = doc.width;
        let height = doc.height;
        let fps = doc.fps;
        let layer_stack = doc.to_layer_stack(doc.active_frame_index);
        let tags = doc.tags.clone();
        Self {
            width,
            height,
            fps,
            palette,
            layer_stack,
            doc,
            tags,
        }
    }

    /// Convenience helper to create a default project.
    pub fn new(width: u32, height: u32, background_color: ColorRgba) -> Self {
        Self::from_layer_stack(LayerStack::new(width, height, background_color), None)
    }
}

/// Serializes and writes a ProjectData to any `Write + Seek` stream as a `.paintproj` ZIP archive.
pub fn save_project_to_writer<W: Write + Seek>(
    project: &ProjectData,
    writer: W,
) -> Result<(), ProjectError> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Save active layer composite image entries (for v1 backward compatibility)
    let mut layer_entries = Vec::with_capacity(project.doc.layers.len());
    let active_frame = project
        .doc
        .frames
        .get(project.doc.active_frame_index)
        .unwrap_or(&project.doc.frames[0]);

    for meta in &project.doc.layers {
        let image_filename = format!("layers/layer_{}.png", meta.id.0);
        let (offset_x, offset_y, buf) = if let Some(cel) = active_frame.get_cel(meta.id) {
            (cel.x, cel.y, &cel.buffer)
        } else {
            (0, 0, &project.layer_stack.layers[0].buffer)
        };

        layer_entries.push(LayerManifestEntry {
            id: meta.id.0,
            name: meta.name.clone(),
            visible: meta.visible,
            locked: meta.locked,
            opacity: meta.opacity,
            blend: meta.blend,
            offset_x,
            offset_y,
            image_path: image_filename.clone(),
        });

        zip.start_file(&image_filename, options)?;
        let png_bytes = save_to_memory(buf, ImageFileFormat::Png)?;
        zip.write_all(&png_bytes)?;
    }

    // Save all frames and cels
    let mut frame_entries = Vec::with_capacity(project.doc.frames.len());
    for (f_idx, frame) in project.doc.frames.iter().enumerate() {
        let mut cel_entries = Vec::new();
        for (layer_id, cel) in &frame.cels {
            let cel_filename = format!("cels/frame_{}_layer_{}.png", f_idx, layer_id.0);
            cel_entries.push(CelManifestEntry {
                layer_id: layer_id.0,
                offset_x: cel.x,
                offset_y: cel.y,
                image_path: cel_filename.clone(),
            });

            zip.start_file(&cel_filename, options)?;
            let png_bytes = save_to_memory(&cel.buffer, ImageFileFormat::Png)?;
            zip.write_all(&png_bytes)?;
        }
        frame_entries.push(FrameManifestEntry {
            duration_ms: frame.duration_ms,
            cels: cel_entries,
        });
    }

    let manifest = ProjectManifest {
        version: PROJECT_VERSION,
        width: project.width,
        height: project.height,
        fps: project.fps,
        palette: project.palette.map(|p| p.to_vec()),
        active_layer_id: project.doc.active_layer_id.0,
        active_frame_index: project.doc.active_frame_index,
        layers: layer_entries,
        frames: frame_entries,
        tags: project
            .doc
            .tags
            .iter()
            .map(|t| AnimTagManifestEntry {
                name: t.name.clone(),
                from_frame: t.from_frame,
                to_frame: t.to_frame,
                color: t.color,
            })
            .collect(),
    };

    zip.start_file("manifest.json", options)?;
    let json_bytes = serde_json::to_vec_pretty(&manifest)?;
    zip.write_all(&json_bytes)?;

    zip.finish()?;
    Ok(())
}

/// Reads and deserializes a ProjectData from any `Read + Seek` stream of a `.paintproj` ZIP archive.
pub fn load_project_from_reader<R: Read + Seek>(reader: R) -> Result<ProjectData, ProjectError> {
    let mut archive = ZipArchive::new(reader)?;

    let manifest: ProjectManifest = {
        let mut manifest_file = archive
            .by_name("manifest.json")
            .map_err(|_| ProjectError::MissingManifest)?;
        let mut json_data = Vec::new();
        manifest_file.read_to_end(&mut json_data)?;
        serde_json::from_slice(&json_data)?
    };

    let mut layer_metas = Vec::with_capacity(manifest.layers.len());
    let mut max_id: u64 = 1;

    for entry in &manifest.layers {
        if entry.id > max_id {
            max_id = entry.id;
        }

        layer_metas.push(LayerMetadata {
            id: LayerId(entry.id),
            name: entry.name.clone(),
            visible: entry.visible,
            locked: entry.locked,
            opacity: entry.opacity,
            blend: entry.blend,
        });
    }

    if layer_metas.is_empty() {
        return Err(ProjectError::InvalidData(
            "Project contains no layers".into(),
        ));
    }

    let active_id = LayerId(manifest.active_layer_id);
    let actual_active_id = if layer_metas.iter().any(|l| l.id == active_id) {
        active_id
    } else {
        layer_metas.first().map(|l| l.id).unwrap_or(LayerId(1))
    };

    let tags: Vec<AnimTag> = manifest
        .tags
        .into_iter()
        .map(|t| {
            let mut tag = AnimTag::new(t.name, t.from_frame, t.to_frame);
            tag.color = t.color;
            tag
        })
        .collect();

    let mut frames = Vec::new();

    if !manifest.frames.is_empty() {
        // Load v2 multi-frame cels
        for frame_entry in &manifest.frames {
            let mut frame = Frame::new(frame_entry.duration_ms);
            for cel_entry in &frame_entry.cels {
                let mut cel_file = archive
                    .by_name(&cel_entry.image_path)
                    .map_err(|_| ProjectError::MissingLayerFile(cel_entry.image_path.clone()))?;

                let mut img_bytes = Vec::new();
                cel_file.read_to_end(&mut img_bytes)?;
                let buffer: PixelBuffer = load_from_memory(&img_bytes)?;

                frame.set_cel(
                    LayerId(cel_entry.layer_id),
                    Cel {
                        x: cel_entry.offset_x,
                        y: cel_entry.offset_y,
                        buffer,
                    },
                );
            }

            // Ensure all layers have a cel in this frame
            for meta in &layer_metas {
                frame.ensure_cel(meta.id, manifest.width, manifest.height);
            }

            frames.push(frame);
        }
    } else {
        // Fallback: load v1 single-frame layers
        let mut first_frame = Frame::new(1000 / manifest.fps.max(1));
        for entry in &manifest.layers {
            let mut layer_file = archive
                .by_name(&entry.image_path)
                .map_err(|_| ProjectError::MissingLayerFile(entry.image_path.clone()))?;

            let mut img_bytes = Vec::new();
            layer_file.read_to_end(&mut img_bytes)?;
            let buffer: PixelBuffer = load_from_memory(&img_bytes)?;

            first_frame.set_cel(
                LayerId(entry.id),
                Cel {
                    x: entry.offset_x,
                    y: entry.offset_y,
                    buffer,
                },
            );
        }
        frames.push(first_frame);
    }

    let active_frame_index = manifest
        .active_frame_index
        .min(frames.len().saturating_sub(1));

    let doc = AnimationDocument {
        width: manifest.width,
        height: manifest.height,
        layers: layer_metas,
        frames,
        tags: tags.clone(),
        fps: manifest.fps,
        active_frame_index,
        active_layer_id: actual_active_id,
        next_layer_id: max_id + 1,
    };

    let layer_stack = doc.to_layer_stack(active_frame_index);

    let palette = manifest.palette.and_then(|p| {
        if p.len() >= 28 {
            let mut arr = [ColorRgba::TRANSPARENT; 28];
            arr.copy_from_slice(&p[..28]);
            Some(arr)
        } else {
            None
        }
    });

    Ok(ProjectData {
        width: manifest.width,
        height: manifest.height,
        fps: manifest.fps,
        palette,
        layer_stack,
        doc,
        tags,
    })
}

/// Serializes a ProjectData to an in-memory byte vector.
pub fn save_project_to_bytes(project: &ProjectData) -> Result<Vec<u8>, ProjectError> {
    let mut cursor = Cursor::new(Vec::new());
    save_project_to_writer(project, &mut cursor)?;
    Ok(cursor.into_inner())
}

/// Deserializes a ProjectData from an in-memory byte slice.
pub fn load_project_from_bytes(bytes: &[u8]) -> Result<ProjectData, ProjectError> {
    load_project_from_reader(Cursor::new(bytes))
}

/// Saves a ProjectData to a `.paintproj` file on disk.
pub fn save_project_to_path<P: AsRef<Path>>(
    project: &ProjectData,
    path: P,
) -> Result<(), ProjectError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    save_project_to_writer(project, writer)?;
    Ok(())
}

/// Loads a ProjectData from a `.paintproj` file on disk.
pub fn load_project_from_path<P: AsRef<Path>>(path: P) -> Result<ProjectData, ProjectError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    load_project_from_reader(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{BLUE, GREEN, RED, WHITE, XP_PALETTE_28};

    #[test]
    fn test_project_manifest_default() {
        let manifest = ProjectManifest::default();
        assert_eq!(manifest.version, 2);
        assert_eq!(manifest.width, 800);
        assert_eq!(manifest.height, 600);
        assert_eq!(manifest.fps, 12);
        assert!(manifest.layers.is_empty());
    }

    #[test]
    fn test_project_save_and_load_roundtrip_memory() {
        let mut stack = LayerStack::new(100, 80, WHITE);
        if let Some(bg) = stack.active_layer_mut() {
            bg.buffer.set_pixel(10, 10, RED);
        }

        let id2 = stack.add_layer("Foreground");
        if let Some(fg) = stack.get_layer_mut(id2) {
            fg.opacity = 75;
            fg.offset_x = 5;
            fg.offset_y = 5;
            fg.buffer.set_pixel(20, 20, BLUE);
            fg.buffer.set_pixel(21, 20, GREEN);
        }

        let mut custom_palette = XP_PALETTE_28;
        custom_palette[0] = ColorRgba::rgb(42, 42, 42);

        let orig_project = ProjectData::from_layer_stack(stack.clone(), Some(custom_palette));
        let orig_composite = stack.composite();

        let bytes = save_project_to_bytes(&orig_project).expect("save_project_to_bytes failed");
        assert!(!bytes.is_empty());

        let loaded_project =
            load_project_from_bytes(&bytes).expect("load_project_from_bytes failed");

        assert_eq!(loaded_project.width, 100);
        assert_eq!(loaded_project.height, 80);
        assert_eq!(loaded_project.fps, 12);
        assert_eq!(loaded_project.palette, Some(custom_palette));

        assert_eq!(loaded_project.layer_stack.layers.len(), 2);
        assert_eq!(loaded_project.layer_stack.layers[0].name, "Background");
        assert_eq!(loaded_project.layer_stack.layers[1].name, "Foreground");
        assert_eq!(loaded_project.layer_stack.layers[1].opacity, 75);
        assert_eq!(loaded_project.layer_stack.layers[1].offset_x, 5);
        assert_eq!(loaded_project.layer_stack.layers[1].offset_y, 5);
        assert_eq!(
            loaded_project.layer_stack.active_layer_id,
            stack.active_layer_id
        );

        let loaded_composite = loaded_project.layer_stack.composite();
        assert_eq!(
            orig_composite.as_raw(),
            loaded_composite.as_raw(),
            "Composite image must be byte-identical after roundtrip!"
        );
    }

    #[test]
    fn test_multi_frame_project_roundtrip() {
        let mut doc = AnimationDocument::new(50, 50, WHITE);
        let l1 = LayerId(1);
        let l2 = doc.add_layer("Anim Layer");

        // Frame 0
        doc.get_cel_mut(0, l1).unwrap().buffer.set_pixel(0, 0, RED);

        // Frame 1
        doc.add_frame(150, false);
        doc.get_cel_mut(1, l2)
            .unwrap()
            .buffer
            .set_pixel(10, 10, BLUE);

        // Frame 2
        doc.add_frame(250, false);
        doc.get_cel_mut(2, l2)
            .unwrap()
            .buffer
            .set_pixel(20, 20, GREEN);

        // Add Tag
        doc.add_tag(AnimTag::new("Loop", 0, 2).with_color(255, 0, 0));

        let project = ProjectData::from_doc(doc.clone(), None);
        let bytes = save_project_to_bytes(&project).unwrap();

        let loaded = load_project_from_bytes(&bytes).unwrap();
        assert_eq!(loaded.doc.frames.len(), 3);
        assert_eq!(loaded.doc.frames[0].duration_ms, 83);
        assert_eq!(loaded.doc.frames[1].duration_ms, 150);
        assert_eq!(loaded.doc.frames[2].duration_ms, 250);
        assert_eq!(loaded.doc.tags.len(), 1);
        assert_eq!(loaded.doc.tags[0].name, "Loop");
        assert_eq!(
            loaded.doc.get_cel(1, l2).unwrap().buffer.get_pixel(10, 10),
            Some(BLUE)
        );
    }

    #[test]
    fn test_corrupted_archive_errors() {
        let invalid_zip = b"Not a zip file content";
        let err = load_project_from_bytes(invalid_zip);
        assert!(err.is_err());

        // Empty zip without manifest
        let mut empty_zip = Cursor::new(Vec::new());
        {
            let zip = ZipWriter::new(&mut empty_zip);
            zip.finish().unwrap();
        }
        let err2 = load_project_from_bytes(&empty_zip.into_inner());
        match err2 {
            Err(ProjectError::MissingManifest) => (),
            other => panic!("Expected MissingManifest, got {:?}", other),
        }
    }
}
