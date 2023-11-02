use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use zip::ZipArchive;

use distribution_filename::WheelFilename;
use platform_tags::Tags;
use puffin_distribution::RemoteDistributionRef;
use puffin_package::pypi_types::Metadata21;

/// A cached wheel built from a remote source.
#[derive(Debug)]
pub(super) struct CachedWheel {
    path: PathBuf,
    filename: WheelFilename,
}

impl CachedWheel {
    pub(super) fn new(path: PathBuf, filename: WheelFilename) -> Self {
        Self { path, filename }
    }

    /// Search for a wheel matching the tags that was built from the given distribution.
    pub(super) fn find_in_cache(
        distribution: &RemoteDistributionRef<'_>,
        tags: &Tags,
        cache: &Path,
    ) -> Option<Self> {
        let wheel_dir = cache.join(distribution.id());
        let Ok(read_dir) = fs_err::read_dir(wheel_dir) else {
            return None;
        };
        for entry in read_dir {
            let Ok(entry) = entry else {
                continue;
            };
            let Ok(filename) =
                WheelFilename::from_str(entry.file_name().to_string_lossy().as_ref())
            else {
                continue;
            };
            if filename.is_compatible(tags) {
                let path = entry.path().clone();
                return Some(CachedWheel { path, filename });
            }
        }
        None
    }

    /// Read the [`Metadata21`] from a wheel.
    pub(super) fn read_dist_info(&self) -> Result<Metadata21> {
        let mut archive = ZipArchive::new(fs_err::File::open(&self.path)?)?;
        let dist_info_prefix = install_wheel_rs::find_dist_info(&self.filename, &mut archive)?;
        let dist_info = std::io::read_to_string(
            archive.by_name(&format!("{dist_info_prefix}.dist-info/METADATA"))?,
        )?;
        Ok(Metadata21::parse(dist_info.as_bytes())?)
    }
}