use std::{
    fs, io,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use axum::{
    Json, Router,
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{AppStateRef, errors::trace_error};

pub fn get_routes(state_ref: &AppStateRef) -> Router {
    Router::new()
        .route("/{id}", get(list_files))
        .route("/{id}/tree", get(list_files))
        .with_state(state_ref.clone())
}

#[derive(Debug, Deserialize)]
struct FileQuery {
    path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
    modified: Option<u64>,
}

fn safe_join(root: &Path, path: Option<&str>) -> Result<PathBuf, StatusCode> {
    let mut joined = root.to_path_buf();
    let Some(path) = path else {
        return Ok(joined);
    };

    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => joined.push(part),
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    Ok(joined)
}

fn to_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .and_then(|p| p.to_str())
        .unwrap_or_default()
        .replace('\\', "/")
}

fn entry_from_dir_entry(root: &Path, entry: fs::DirEntry) -> io::Result<FileEntry> {
    let path = entry.path();
    let metadata = entry.metadata()?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());

    Ok(FileEntry {
        name: entry.file_name().to_string_lossy().into_owned(),
        path: to_relative(root, &path),
        is_dir: metadata.is_dir(),
        size: metadata.len(),
        modified,
    })
}

#[instrument(skip(state))]
async fn list_files(
    State(state): State<AppStateRef>,
    AxumPath(id): AxumPath<i32>,
    Query(query): Query<FileQuery>,
) -> Result<Json<Vec<FileEntry>>, StatusCode> {
    let root = state.instances_path.join(id.to_string());
    fs::create_dir_all(&root).map_err(trace_error!(
        "create instance file root",
        StatusCode::INTERNAL_SERVER_ERROR
    ))?;

    let path = safe_join(&root, query.path.as_deref())?;
    let metadata = fs::metadata(&path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => StatusCode::NOT_FOUND,
        _ => trace_error!("read file metadata", StatusCode::INTERNAL_SERVER_ERROR)(error),
    })?;

    if !metadata.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut entries = fs::read_dir(&path)
        .map_err(trace_error!("read dir", StatusCode::INTERNAL_SERVER_ERROR))?
        .map(|entry| entry.and_then(|entry| entry_from_dir_entry(&root, entry)))
        .collect::<Result<Vec<_>, _>>()
        .map_err(trace_error!(
            "build file entries",
            StatusCode::INTERNAL_SERVER_ERROR
        ))?;

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

    Ok(Json(entries))
}
