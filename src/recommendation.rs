use std::ops::Mul;

use crate::prelude::*;
use nalgebra_sparse::{coo::CooMatrix, CsrMatrix, CscMatrix};


pub fn compute_matrix(similar_users: Vec<(UID, HashMap<MID, u8>)>, me: HashMap<MID, u8>, n_movies: usize) -> CooMatrix<f32> {
    let unique_movies: HashSet<MID> = HashSet::from_iter(
        similar_users
            .iter()
            .flat_map(|(_, movies)| movies.keys().cloned()),
    );

    let mut matrix = CooMatrix::new(similar_users.len() + 1, unique_movies.len());

    // Add the users
    for (i, (user, movies)) in similar_users.iter().enumerate() {
        for (j, movie) in unique_movies.iter().enumerate() {
            if let Some(rating) = movies.get(movie) {
                matrix.push(i, j, *rating as f32);
            }
        }
    }

    // Add me
    for (j, movie) in unique_movies.iter().enumerate() {
        if let Some(rating) = me.get(movie) {
            matrix.push(similar_users.len(), j, *rating as f32);
        }
    }
    matrix
}

pub fn normalize_matrix(matrix: &mut CsrMatrix<f32>) {
    for mut raw in matrix.row_iter_mut() {
        // Mean centering
        let values = raw.values();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        for value in raw.values_mut() {
            if value != &0.0 {
                *value -= mean;
            }
        }
    }
}

pub fn compute_reco(matrix: CsrMatrix<f32>, simarities: CsrMatrix<f32>) -> CsrMatrix<f32> {
    // Simularities mult Matrix
    let sum_similarities: Vec<f32> = CscMatrix::from(&simarities).col_iter().map(|col| col.values().iter().sum::<f32>()).collect();
    let mut output = simarities.mul(matrix);
    for mut raw in output.row_iter_mut() {
        for (i, value) in raw.values_mut().iter_mut().enumerate() {
            *value /= sum_similarities[i];
        }
    }
    output
}


pub async fn get_recommendations(
    matrix: &CsrMatrix<f32>,
    query: &RecoQuery,
) -> Vec<(MID, u8)> {
    // Get all similar users
    let mut similar_users: HashMap<UID, HashMap<MID, u8>> = HashMap::new();
    for movie in &query.ratings {
        if let Some(users) = recos.get(movie.0) {
            for user in users {
                if query.ratings.contains_key(movie.0) {
                    similar_users
                        .entry(user.0.to_owned())
                        .or_insert(HashMap::new())
                        .insert(*movie.0, *movie.1);
                }
            }
        }
    }
    // Get the most top k similar users to the query user
    let k = 100;
    let mut top_users: Vec<(UID, HashMap<MID, u8>)> = Vec::with_capacity(k);
    for user in similar_users {
        let similarity = get_similarity(user.1.clone(), query.ratings.clone());
        if top_users.len() < k {
            top_users.push((user.0, user.1));
        } else {
            let mut min = 0;
            for i in 0..top_users.len() {
                if get_similarity(top_users[i].1.clone(), query.ratings.clone()) < similarity {
                    min = i;
                }
            }
            top_users[min] = (user.0, user.1);
        }
    }

    let now = std::time::Instant::now();
    let matrix = compute_matrix(top_users, query.ratings.clone());
    let mut matrix = CsrMatrix::from(&matrix);
    normalize_matrix(&mut matrix);
    let similarities = computer_similarities(matrix.clone());
    // To matrix
    let mut similarities_matrix = CooMatrix::new(1, similarities.len());
    for (i, similarity) in similarities.iter().enumerate() {
        similarities_matrix.push(0, i, *similarity);
    }
    println!("Recos {:?}", compute_reco(matrix, CsrMatrix::from(&similarities_matrix)));
    println!("Time to compute similarities: {:?}", now.elapsed());

    Vec::new()
}


#[cfg(test)]
mod tests {
    use nalgebra_sparse::{CsrMatrix, CooMatrix};

    use crate::prelude::compute_reco;


    #[test]
    fn test_matrix_similarities() {

    }
}