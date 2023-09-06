use std::ops::Mul;

use crate::prelude::*;
use nalgebra_sparse::{coo::CooMatrix, CsrMatrix, CscMatrix, na::{RowVector, Similarity}};


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
) -> Vec<(MID, u8)> {
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
            if value != &0.0 {
                *value /= norms[to_predict] * norms[i];
            } else {
                *value = -1.0;
            }            

            if i == to_predict {
                *value = -1.0;
            }
            
            if n < K {
                top_k[n] = (*value, i);
                n += 1;
            } else {
                let mut min = 0;
                for (j, (v, _)) in top_k.iter().enumerate() {
                    if v < value {
                        min = j;
                    }
                }
                top_k[min] = (*value, i);
            }
        }
    }

    // Build the matrix with the top k similar users
    println!("building the matrix with the top k similar users");
    // Get the movies that the user hasn't seen
    println!("{:?}", top_k);
   

    Vec::new()
}

