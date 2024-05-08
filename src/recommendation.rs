use std::ops::Mul;
use crate::prelude::*;
use nalgebra_sparse::{coo::CooMatrix, CscMatrix, CsrMatrix, SparseEntryMut};



// pub fn compute_matrix(similar_users_rates: Vec<(UID, HashMap<MID, f32>)>, me: HashMap<MID, f32>, n_movies: usize) -> CooMatrix<f32> {
//     let unique_movies: HashSet<MID> = HashSet::from_iter(
//         similar_users_rates
//             .iter()
//             .flat_map(|(_, movies)| movies.keys().cloned()),
//     );

//     let mut matrix = CooMatrix::new(similar_users_rates.len() + 1, unique_movies.len());

//     // Add the users
//     for (i, (user, movies)) in similar_users_rates.iter().enumerate() {
//         for (j, movie) in unique_movies.iter().enumerate() {
//             if let Some(rating) = movies.get(movie) {
//                 matrix.push(i, j, *rating);
//             }
//         }
//     }

//     // Add me
//     for (j, movie) in unique_movies.iter().enumerate() {
//         if let Some(rating) = me.get(movie) {
//             matrix.push(similar_users_rates.len(), j, *rating);
//         }
//     }
//     matrix
// }

pub fn normalize_matrix(matrix: &mut CsrMatrix<f32>) {
    for raw in matrix.row_iter_mut() {
        // Mean centering
        /*let values = raw.values();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        for value in raw.values_mut() {
            if value != &0.0 {
                *value -= mean;
            }
        }*/
    }
}

pub fn compute_norms(matrix: &CsrMatrix<f32>) -> Vec<f32> {
    matrix
        .row_iter()
        .map(|row| row.values().iter().map(|v| v.powi(2)).sum::<f32>().sqrt())
        .collect()
}

pub async fn get_recommendations(
    matrix: &CsrMatrix<f32>,
    norms: &[f32],
    to_predict: usize,
) -> Vec<(MID, f32)> {
    let mut norms = norms.to_vec();
    // Take a row from the matrix and convert it to a Matrix
    println!("computing the row");
    let to_predict_row = matrix.row(to_predict);
    let mut user_matrix: CooMatrix<f32> = CooMatrix::new(1, to_predict_row.ncols());
    let l = to_predict_row.values().len() / 2;
    let mut a = 0.0;
    for (i, ind) in to_predict_row.col_indices().iter().enumerate() {
        let value = to_predict_row.get_entry(*ind).unwrap().into_value();
        user_matrix.push(0, *ind, value);
        
        a += value * value;
        if i >= l  {
            norms[to_predict] = a.sqrt();
            break;
        }
    }

    let user_matrix = CsrMatrix::from(&user_matrix);

    // Compute the cosine similarity between the user and the other users
    println!("computing the cosine similarity");
    let mut sim = matrix.mul(&user_matrix.transpose());

    // Divide by the norms and get the top k 
    println!("dividing by the norms");
    // Get the top k of similar users
    const K: usize = 10000;
    let mut top_k = Vec::new();

    for (i, mut row) in sim.row_iter_mut().enumerate() {
        let mut total_similarity = 0.0;
        for value in row.values_mut() {
            if norms[i] == 0.0 {
                continue;
            }
            *value /= norms[to_predict] * norms[i];    

            if i == to_predict {
                *value = 0.0;
                continue;
            }

            total_similarity += *value;
        }

        if total_similarity > 0.0 {
            top_k.push((total_similarity, i));
        }
    }
    // Keep only the top K similar users
    top_k.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    top_k.truncate(K);

    // Build the matrix with the top k similar users
    println!("building the matrix with the top k similar users");
    let mut similar_users_rates = CooMatrix::new(K, matrix.ncols()); // The matrix with the top k similar users of the shape (K, n_movies)
    let mut similarities_users = CooMatrix::new(1, K); // The matrix with the similarities between the user to recommand and each other user of the shape (1, K)
    // Put the data in the matrices
    for (i, (similarity, user)) in top_k.iter().enumerate() {
        for (j, v) in matrix.row(*user).values().iter().enumerate() {
            similar_users_rates.push(i, matrix.row(*user).col_indices()[j], *v);
        }

        similarities_users.push(0, i, *similarity);
    }

    // Convert the matrices to CscMatrix
    let similar_users_rates = CscMatrix::from(&similar_users_rates);
    let similarities_users = CscMatrix::from(&similarities_users);

    println!("computing the weighted sum");
    let mut estimation = similarities_users.clone().mul(&(similar_users_rates.clone()));

    // Compute the sum of the similarities with the following part of the fomula: sum(similarities_users * similar_users_rates)
    for movie_id in 0..matrix.ncols() {
        let mut sum = 0.0;
        let col = similar_users_rates.col(movie_id);
        // Check if the movie has been rated by the user
        for (i, value) in col.values().iter().enumerate() { 
            sum += similarities_users.get_entry(0, i).unwrap().into_value() * value;
        }
        
        let entry = estimation.get_entry_mut(0, movie_id).unwrap();
        if let SparseEntryMut::NonZero(entry) = entry {
            *entry /= sum;
        } 
    }

    println!("getting the movies that the user has not seen yet and sort them by the predicted rating");
    let mut movies = Vec::new();
    for c in 0..estimation.ncols() {
        let value = estimation.get_entry(0, c).unwrap().into_value();
        if value > 0.0 {
            movies.push((c, value));
        }
    }
    movies.sort_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());
    movies
}

