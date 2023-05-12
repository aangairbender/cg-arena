use serde::{Serialize, Deserialize};
use skillratings::weng_lin::WengLinRating;

#[derive(Serialize, Deserialize, Clone)]
pub struct Bot {
    pub name: String,
    pub description: String,
    pub language_name: String,
    pub completed_matches: u32,
    pub raw_rating: WengLinRating,
}

impl Bot {
    pub fn new(name: String, description: String, language_name: String) -> Self {
        Self {
            name,
            description,
            language_name,
            completed_matches: 0,
            raw_rating: Default::default(),
        }
    }

    pub fn rating(&self) -> f64 {
        self.raw_rating.rating - self.raw_rating.uncertainty * 3.0
    }
}
