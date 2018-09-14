#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Rating {
    Safe,
    Questionable,
    Explicit,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum AllowedRating {
    Only(Rating),
    Above(Rating),
    Below(Rating),
    All,
}

impl AllowedRating {
    pub fn allows(&self, rating: &Rating) -> bool {
        use self::Rating;
        use self::AllowedRating;

        match self {
            Only(r) => r == rating,

            Above(r) => match r {
                Safe => rating == Safe,
                Questionable => rating == Safe || rating == Questionable,
                Explicit => true,
            },

            Below(r) => match r {
                Safe => true,
                Questionable => rating == Questionable || rating == Explicit,
                Explicit => rating == Explicit,
            },

            All => true,
        }
    }
}

pub struct Config {
    pub ratio: f64,
    pub tolerance: f64,
    pub allowed_rating: AllowedRating,
    pub location: Path,
}

impl Default for Config {
    fn default() -> Config {
        let rating = AllowedRating::Above(Rating::Questionable),

        Config {
            ratio: 16./9.,
            tolerance: 1. / 1024.,
            allowed_rating: rating,
            location: Path::new(".").clone(),
        }
    }
}

impl Config {
    pub fn is_tolerated_aspect_ratio(width: usize, height: usize) -> bool {
        let aspect_ratio =  width as f64 / height as f64;
        let difference = aspect_ratio - ratio;

        // -tolerance < difference < tolerance
        -tolerance <= difference && difference <= tolerance
    }
}
