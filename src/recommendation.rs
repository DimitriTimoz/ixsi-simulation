use std::ops::Mul;

use crate::prelude::*;
use nalgebra_sparse::{coo::CooMatrix, CsrMatrix, CscMatrix};


pub fn compute_matrix(similar_users_rates: Vec<(UID, HashMap<MID, u8>)>, me: HashMap<MID, u8>, n_movies: usize) -> CooMatrix<f32> {
    let unique_movies: HashSet<MID> = HashSet::from_iter(
        similar_users_rates
            .iter()
            .flat_map(|(_, movies)| movies.keys().cloned()),
    );

    let mut matrix = CooMatrix::new(similar_users_rates.len() + 1, unique_movies.len());

    // Add the users
    for (i, (user, movies)) in similar_users_rates.iter().enumerate() {
        for (j, movie) in unique_movies.iter().enumerate() {
            if let Some(rating) = movies.get(movie) {
                matrix.push(i, j, *rating as f32);
            }
        }
    }

    // Add me
    for (j, movie) in unique_movies.iter().enumerate() {
        if let Some(rating) = me.get(movie) {
            matrix.push(similar_users_rates.len(), j, *rating as f32);
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
    // Take a row from the matrix and convert it to a Matrix
    println!("computing the row");
    let row = matrix.row(to_predict);
    let mut user_matrix: CooMatrix<f32> = CooMatrix::new(1, row.ncols());
    for ind in row.col_indices() {
        let value = row.get_entry(*ind).unwrap().into_value();
        user_matrix.push(0, *ind, value);
    }

    let user_matrix = CsrMatrix::from(&user_matrix);

    // Compute the cosine similarity between the user and the other users
    println!("computing the cosine similarity");
    let mut sim = matrix.mul(&user_matrix.transpose());

    // Divide by the norms and get the top k 
    println!("dividing by the norms");
    // Get the top k of similar users
    const K: usize = 100;
    let mut top_k = [(0.0, 0); K];
    let mut n = 0;
  
    for (i, mut row) in sim.row_iter_mut().enumerate() {
        for value in row.values_mut() {
            // Divide by the norms to get the cosine similarity 
            if norms[i] == 0.0 {
                continue;
            }
            *value /= norms[to_predict] * norms[i];    
            
            if i == to_predict {
                *value = -1.0;
                continue;
            }

            // Get the top k of most similar users
            if n < K {
                top_k[n] = (*value, i);
                n += 1;
            } else {
                let mut min = 0;
           
                for (j, (v, _)) in top_k.iter().enumerate() {
                    if v < &top_k[min].0 {
                        min = j;
                    }
                }
                top_k[min] = (*value, i);
            }
        }
    }
    // Build the matrix with the top k similar users
    println!("building the matrix with the top k similar users");
    let mut similar_users_rates = CooMatrix::new(K, matrix.ncols()); // The matrix with the top k similar users of the shape (K, n_movies)
    let mut similarities_users = CooMatrix::new(1, K); // The matrix with the similarities between the user to recommand and each other user of the shape (1, K)
    // Put the data in the matrices
    for (i, (similarity, user)) in top_k.iter().enumerate() {
        similarities_users.push(0, i, *similarity);
        for (j, v) in matrix.row(*user).values().iter().enumerate() {
            similar_users_rates.push(i, j, *v);
        }
    }

    // Convert the matrices to CscMatrix
    let similar_users_rates = CscMatrix::from(&similar_users_rates);
    let similarities_users = CscMatrix::from(&similarities_users);
    
    // Compute the sum of the similarities with the following part of the fomula: sum(similarities_users * similar_users_rates)
    let mut estimation = similarities_users.clone().mul(&(similar_users_rates.clone())); // Of the shape (1, n_movies)
    for (i, mut col) in estimation.col_iter_mut().enumerate() {
        // Get the column i of the similar_users_rates matrix and do the sum
        let mut s = 0.0;
        for v in similar_users_rates.get_col(i).unwrap().values().iter() {
            s += v.abs();
        }

        for value in col.values_mut() {
            *value /= s;
        }
    }
    
    // Get the movies that the user has not seen yet and sort them by the predicted rating
    println!("getting the movies that the user has not seen yet and sort them by the predicted rating");
    let mut movies = Vec::new();
    for c in 0..estimation.ncols() {
        let value = estimation.get_entry(0, c).unwrap().into_value();
        if value > 0.0 {
            movies.push((c, value));
        }
    }
    movies.sort_by(|(_, v1), (_, v2)| v2.partial_cmp(v1).unwrap());
    println!("{:?}", movies);
    movies
}

