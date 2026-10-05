// This is a source heuristic, not proof of screenshot provenance.
pub fn accepted(owner: &str) -> bool {
    matches!(
        owner.to_ascii_lowercase().as_str(),
        "screenclippinghost.exe" | "snippingtool.exe" | "screensketch.exe"
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_owners() {
        assert!(accepted("ScreenClippingHost.exe"));
        assert!(accepted("SnippingTool.exe"));
    }
    #[test]
    fn ordinary_and_unknown_owners() {
        for x in [
            "",
            "chrome.exe",
            "photoshop.exe",
            "explorer.exe",
            "FakeScreenClippingHost.exe",
        ] {
            assert!(!accepted(x));
        }
    }
}

/// Observed on Sammer's build: a clipboard broker owns snip data.
/// This is intentionally labelled a candidate, not definitive provenance.
pub fn classify(owner: &str, formats: &[String], broker_candidates: bool) -> Option<&'static str> {
    if accepted(owner) {
        return Some("saved_owner_match");
    }
    let has = |name: &str| {
        formats
            .iter()
            .any(|f| f.split_once(':').map(|(_, n)| n == name).unwrap_or(false))
    };
    let bitmap = formats
        .iter()
        .any(|f| matches!(f.as_str(), "2" | "8" | "17"));
    if broker_candidates
        && owner.eq_ignore_ascii_case("svchost.exe")
        && bitmap
        && has("PNG")
        && has("DataObject")
        && has("Preferred DropEffect")
    {
        Some("saved_broker_png_candidate")
    } else {
        None
    }
}

#[cfg(test)]
pub fn fingerprint(pixels: &[u8], width: i32, height: i32) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    width.hash(&mut h);
    height.hash(&mut h);
    pixels.hash(&mut h);
    h.finish()
}
#[cfg(test)]
pub fn duplicate(recent: &[(u64, u128)], hash: u64, time: u128) -> bool {
    recent
        .iter()
        .any(|(h, t)| *h == hash && time.saturating_sub(*t) <= 2000)
}
#[cfg(test)]
mod broker_tests {
    use super::*;
    fn signature() -> Vec<String> {
        [
            "49161:DataObject",
            "49311:Preferred DropEffect",
            "2",
            "49397:PNG",
            "8",
            "17",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
    #[test]
    fn recognizes_observed_broker_signature() {
        assert_eq!(
            classify("svchost.exe", &signature(), true),
            Some("saved_broker_png_candidate")
        );
    }
    #[test]
    fn broker_is_not_blanket_allowlisted() {
        assert_eq!(classify("svchost.exe", &[], true), None);
        assert_eq!(classify("chrome.exe", &signature(), true), None);
        assert_eq!(classify("svchost.exe", &signature(), false), None);
        let mut f = signature();
        f.retain(|s| !s.ends_with(":PNG"));
        assert_eq!(classify("svchost.exe", &f, true), None);
    }
    #[test]
    fn ignores_dynamic_format_ids() {
        let f = [
            "60001:DataObject",
            "60002:Preferred DropEffect",
            "2",
            "60003:PNG",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        assert!(classify("svchost.exe", &f, true).is_some());
    }
    #[test]
    fn duplicate_window_expires_and_dimensions_matter() {
        let h = fingerprint(&[1, 2, 3], 1, 1);
        assert!(duplicate(&[(h, 1000)], h, 1500));
        assert!(!duplicate(&[(h, 1000)], h, 3001));
        assert!(!duplicate(
            &[(h, 1000)],
            fingerprint(&[1, 2, 3], 3, 1),
            1500
        ));
    }
}

/// Virtual-key sequence for the native Windows capture shortcut. Release in reverse order.
pub fn capture_shortcut() -> [(u16, bool); 6] {
    [
        (0x5b, false),
        (0x10, false),
        (0x53, false),
        (0x53, true),
        (0x10, true),
        (0x5b, true),
    ]
}
#[cfg(test)]
mod shortcut_tests {
    #[test]
    fn shortcut_balances_keys_and_releases_in_reverse_order() {
        let keys = super::capture_shortcut();
        assert_eq!(keys[..3], [(0x5b, false), (0x10, false), (0x53, false)]);
        for i in 0..3 {
            assert_eq!(keys[i].0, keys[5 - i].0);
            assert!(keys[5 - i].1);
        }
    }
}

#[derive(Default)]
pub struct Dismissal {
    closed_at: Option<u64>,
}
impl Dismissal {
    pub fn observed(&self) -> bool {
        self.closed_at.is_some()
    }
    pub fn tick(&mut self, visible: bool, now_ms: u64, clipboard_busy: bool) -> bool {
        if visible {
            self.closed_at = None;
            return false;
        }
        let since = *self.closed_at.get_or_insert(now_ms);
        !clipboard_busy && now_ms.saturating_sub(since) >= 750
    }
}
#[cfg(test)]
mod dismissal_tests {
    use super::*;
    #[test]
    fn waits_for_clipboard_grace() {
        let mut d = Dismissal::default();
        assert!(!d.tick(false, 1000, false));
        assert!(!d.tick(false, 1749, false));
        assert!(d.tick(false, 1750, false));
    }
    #[test]
    fn clipboard_busy_extends_grace() {
        let mut d = Dismissal::default();
        d.tick(false, 0, false);
        assert!(!d.tick(false, 1000, true));
        assert!(d.tick(false, 1001, false));
    }
    #[test]
    fn window_reappearing_resets_close_observation() {
        let mut d = Dismissal::default();
        d.tick(false, 0, false);
        assert!(!d.tick(true, 1000, false));
        assert!(!d.observed());
        assert!(!d.tick(false, 2000, false));
        assert!(d.tick(false, 2750, false));
    }
}
