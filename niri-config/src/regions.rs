use crate::utils::{FloatOrInt, MergeWith};

/// The final parsed regions configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Regions {
    pub enabled: bool,
    pub width_percent: f64,
    pub slant_percent: f64,
    pub gap: f64,
}

impl Default for Regions {
    fn default() -> Self {
        Self {
            enabled: false,
            width_percent: 46.0,
            slant_percent: 12.0,
            gap: 16.0,
        }
    }
}

/// A single part of the regions configuration parsed from one config file.
#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct RegionsPart {
    #[knuffel(child, unwrap(argument))]
    pub enabled: Option<bool>,
    #[knuffel(child, unwrap(argument))]
    pub width_percent: Option<FloatOrInt<0, 100>>,
    #[knuffel(child, unwrap(argument))]
    pub slant_percent: Option<FloatOrInt<-100, 100>>,
    #[knuffel(child, unwrap(argument))]
    pub gap: Option<FloatOrInt<0, 1000>>,
}

impl MergeWith<RegionsPart> for Regions {
    fn merge_with(&mut self, part: &RegionsPart) {
        merge_clone!((self, part), enabled);
        merge!((self, part), width_percent, slant_percent, gap);
    }
}

/// Validates that the regions configuration is sane.
/// Invalid values fall back to defaults without crashing.
impl Regions {
    pub fn validate(&mut self) {
        self.width_percent = self.width_percent.clamp(0.0, 100.0);
        self.slant_percent = self.slant_percent.clamp(-100.0, 100.0);
        self.gap = self.gap.clamp(0.0, 1000.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_regions() {
        let regions = Regions::default();
        assert!(!regions.enabled);
        assert_eq!(regions.width_percent, 46.0);
        assert_eq!(regions.slant_percent, 12.0);
        assert_eq!(regions.gap, 16.0);
    }

    #[test]
    fn regions_validation() {
        let mut regions = Regions {
            enabled: true,
            width_percent: 150.0,
            slant_percent: -150.0,
            gap: 2000.0,
        };
        regions.validate();
        assert_eq!(regions.width_percent, 100.0);
        assert_eq!(regions.slant_percent, -100.0);
        assert_eq!(regions.gap, 1000.0);
    }

    #[test]
    fn regions_parsing() {
        let text = r#"
            regions {
                enabled true
                width-percent 50
                slant-percent 15
                gap 20
            }
        "#;
        let config = crate::Config::parse_mem(text).unwrap();
        assert!(config.regions.enabled);
        assert_eq!(config.regions.width_percent, 50.0);
        assert_eq!(config.regions.slant_percent, 15.0);
        assert_eq!(config.regions.gap, 20.0);
    }
}
