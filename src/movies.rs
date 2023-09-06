use std::hash::Hash;

use crate::prelude::*;
use nalgebra_sparse::CooMatrix;
use rand;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Movie {
    pub id: usize,
    pub rating: u8,
}

pub type MID = usize;
pub type UID = usize;

impl Movie {
    pub fn new(id: usize, rating: u8) -> Self {
        Self { id, rating }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct UserRatings {
    pub user_id: usize,
    pub ratings: HashSet<Movie>,
}

#[derive(Clone, Debug)]
pub struct RecoQuery {
    pub user_id: usize,
    pub ratings: HashMap<MID, u8>,
}

impl From<UserRatings> for RecoQuery {
    fn from(user_ratings: UserRatings) -> Self {
        Self {
            ratings: user_ratings
                .ratings
                .iter()
                .map(|movie| (movie.id, movie.rating))
                .collect(),
            user_id: user_ratings.user_id,
        }
    }
}

impl From<Vec<Movie>> for RecoQuery {
    fn from(ratings: Vec<Movie>) -> Self {
        Self {
            ratings: ratings
                .iter()
                .map(|movie| (movie.id, movie.rating))
                .collect(),
            user_id: 0,
        }
    }
}

impl RecoQuery {
    pub fn get_ratings_hashmap(&self) -> HashMap<usize, u8> {
        self.ratings.clone()
    }
}

pub fn get_matrix_and_ratings() -> (
    CooMatrix<f32>,
    HashMap<UID, HashMap<MID, u8>>,
    [usize; 10],
) {
    let mut ratings_count = [0; 10];
    // Read the lines
    let file = read_to_string("ratings.csv")
        .expect("Failed to open file: ratings_small.csv seems doesn't exist");
    let mut matrix =  CooMatrix::new(270_896, 45_115);
    let mut user_ratings = HashMap::new();

    let n_users = 300;

    for line in file.lines().skip(1) {
        let mut line = line.split(',');
        let user_id = line.next().unwrap().parse::<UID>().unwrap() - 1;
        let movie_id = line.next().unwrap().parse::<MID>().unwrap() - 1;
        let rating = line.next().unwrap().parse::<f32>().unwrap();
        let rating = (rating * 2.0).round() as u8;
        ratings_count[rating as usize - 1] += 1;
        let movie = Movie::new(movie_id, rating);
        // Add the movie to the user's list of movies if is selected
        if user_id > n_users {
            continue;
        }

        matrix.push(user_id, movie_id, rating as f32 / 5.0);

        // Add the movie to the user's list of movies if is selected
        user_ratings
            .entry(user_id)
            .or_insert_with(HashMap::new)
            .insert(movie_id, rating);
    }

    (
        matrix,
        user_ratings,
        ratings_count,
    )
}
