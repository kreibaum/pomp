use std::fmt::Display;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct PlayerUuid(String);

impl Display for PlayerUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PlayerUuid {
    pub fn from_query_string(query_string: &str) -> Option<Self> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "UUID=([0-9A-F]{8}-[0-9A-F]{4}-4[0-9A-F]{3}-[89AB][0-9A-F]{3}-[0-9A-F]{12})"
            )
            .unwrap();
        }

        if let Some(cap) = RE.captures_iter(&query_string.to_uppercase()).next() {
            if let Some(uuid) = cap.get(1) {
                return Some(PlayerUuid(uuid.as_str().to_owned()));
            }
        }

        None
    }
}
