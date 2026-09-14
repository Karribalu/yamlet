pub const TEMPLATES_DIR: &str = "terraform/templates";
pub enum SupportKind {
    Infra,
    App,
}
impl SupportKind {
    pub fn variants() -> [&'static str; 2] {
        ["Infra", "App"]
    }

    pub fn is_valid(kind: &str) -> bool {
        Self::variants().contains(&kind)
    }
}

pub enum SupportCloud {
    AWS,
    Azure,
    GCP,
}
impl SupportCloud {
    pub fn variants() -> [&'static str; 3] {
        ["AWS", "Azure", "GCP"]
    }

    pub fn is_valid(cloud: &str) -> bool {
        Self::variants().contains(&cloud)
    }
}
