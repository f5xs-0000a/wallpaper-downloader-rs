use std::path::{
    Path,
    PathBuf,
};

use rating::{
    AllowedRating,
    Rating,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Config {
    pub ratio: f64,
    pub tolerance: f64,
    pub allowed_rating: AllowedRating,
    pub location: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        let rating = AllowedRating::Above(Rating::Questionable);

        Config {
            ratio: 16. / 9.,
            tolerance: 1. / 1024.,
            allowed_rating: rating,
            location: Path::new(".").to_path_buf(),
        }
    }
}

impl Config {
    pub fn is_tolerated_aspect_ratio(
        &self,
        width: usize,
        height: usize,
    ) -> bool {
        let aspect_ratio = width as f64 / height as f64;
        let difference = aspect_ratio - self.ratio;

        // the difference must satisfy a condition such that
        // -tolerance < difference < tolerance
        -self.tolerance <= difference && difference <= self.tolerance
    }
}
