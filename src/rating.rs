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
        use self::Rating::*;
        use self::AllowedRating::*;

        match self {
            Only(r) => r == rating,

            Above(r) => match r {
                Safe => *rating == Safe,
                Questionable => *rating == Safe || *rating == Questionable,
                Explicit => true,
            },

            Below(r) => match r {
                Safe => true,
                Questionable => *rating == Questionable || *rating == Explicit,
                Explicit => *rating == Explicit,
            },

            All => true,
        }
    }
}
