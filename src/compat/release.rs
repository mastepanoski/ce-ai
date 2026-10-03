use std::cmp::Ordering;

/// Strongly-typed representation of an upstream Compound Engineering release tag.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CeRelease {
    pub tag: String,
    pub version: String,
}

impl CeRelease {
    /// Parses a raw tag (e.g. "v1.2.3", "1.2.3", or "compound-engineering-v1.2.3")
    /// into a structured `CeRelease`.
    pub fn parse_tag(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }

        let version_part = if let Some(stripped) = trimmed.strip_prefix("compound-engineering-v") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("compound-engineering-") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix('v') {
            stripped
        } else {
            trimmed
        };

        // Validate basic semver structure X.Y.Z
        let segments: Vec<&str> = version_part.split('.').collect();
        if segments.is_empty() || segments.len() > 4 {
            return None;
        }
        for seg in &segments {
            if seg.is_empty() || !seg.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
        }

        Some(Self {
            tag: trimmed.to_string(),
            version: version_part.to_string(),
        })
    }

    /// Compares two release versions numerically (e.g. 1.10.0 > 1.9.0).
    pub fn compare_version(&self, other: &Self) -> Ordering {
        let self_parts: Vec<u64> = self
            .version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let other_parts: Vec<u64> = other
            .version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        self_parts.cmp(&other_parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags() {
        let r1 = CeRelease::parse_tag("v1.2.3").unwrap();
        assert_eq!(r1.version, "1.2.3");

        let r2 = CeRelease::parse_tag("compound-engineering-v2.10.4").unwrap();
        assert_eq!(r2.version, "2.10.4");

        let r3 = CeRelease::parse_tag("0.9.1").unwrap();
        assert_eq!(r3.version, "0.9.1");

        assert!(CeRelease::parse_tag("").is_none());
        assert!(CeRelease::parse_tag("invalid_tag").is_none());
    }

    #[test]
    fn test_compare_versions() {
        let r_low = CeRelease::parse_tag("v1.9.0").unwrap();
        let r_high = CeRelease::parse_tag("v1.10.0").unwrap();
        assert_eq!(r_low.compare_version(&r_high), Ordering::Less);
        assert_eq!(r_high.compare_version(&r_low), Ordering::Greater);
        assert_eq!(r_low.compare_version(&r_low), Ordering::Equal);
    }
}
