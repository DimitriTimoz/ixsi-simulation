use std::hash::Hash;

use crate::prelude::*;
use nalgebra_sparse::CooMatrix;

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
    HashMap<UID, HashMap<MID, f32>>,
    Vec<f32>,
) {
    let mut ratings_count = [0; 176_275];
    let mut ratings_sum = [0.0; 176_275];
    // Read the lines
    let file = read_to_string("ratings.csv")
        .expect("Failed to open file: ratings_small.csv seems doesn't exist");
    let mut matrix =  CooMatrix::new(1_000_000, 176_275);
    let mut user_ratings = HashMap::new();


    for line in file.lines().skip(1) {
        let mut line = line.split(',');
        let user_id = line.next().unwrap().parse::<UID>().unwrap() - 1;
        let movie_id = line.next().unwrap().parse::<MID>().unwrap() - 1;
        let rating = line.next().unwrap().parse::<f32>().unwrap();
        
        ratings_count[movie_id] += 1;
        ratings_sum[movie_id] += rating;

        // Add the movie to the user's list of movies if is selected
        matrix.push(user_id, movie_id, rating / 5.0);
        if user_id >= 1_000_000 - 1 {
            break;
        }
        // Add the movie to the user's list of movies if is selected
        user_ratings
            .entry(user_id)
            .or_insert_with(HashMap::new)
            .insert(movie_id, rating);
    }

    for (sum, count) in ratings_sum.iter_mut().zip(ratings_count.iter()) {
        if *count == 0 {
            continue;
        }
        *sum /= *count as f32;
    }
    
    (
        matrix,
        user_ratings,
        ratings_sum.to_vec(),
    )
}
