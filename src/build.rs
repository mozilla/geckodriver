/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use serde_json::Value;
use std::fmt;

include!(concat!(env!("OUT_DIR"), "/build-info.rs"));

pub struct BuildInfo;

impl BuildInfo {
    pub fn version() -> &'static str {
        crate_version!()
    }

    pub fn hash() -> Option<&'static str> {
        COMMIT_HASH
    }

    pub fn date() -> Option<&'static str> {
        COMMIT_DATE
    }
}

impl fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BuildInfo::version())?;
        match (BuildInfo::hash(), BuildInfo::date()) {
            (Some(hash), Some(date)) => write!(f, " ({} {})", hash, date)?,
            (Some(hash), None) => write!(f, " ({})", hash)?,
            _ => {}
        }
        Ok(())
    }
}

impl From<BuildInfo> for Value {
    fn from(_: BuildInfo) -> Value {
        Value::String(BuildInfo::version().to_string())
    }
}

/// Returns build-time information about geckodriver.
pub fn build_info() -> BuildInfo {
    BuildInfo {}
}
