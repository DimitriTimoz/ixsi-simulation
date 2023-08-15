use std::ops::Mul;

use crate::prelude::*;
use nalgebra_sparse::{coo::CooMatrix, CsrMatrix, csr::CsrRow, CscMatrix};

fn get_cosine(vec1: Vec<f32>, vec2: Vec<f32>, vec1_mean: f32, vec2_mean: f32) -> f32 {
    assert!(vec1.len() == vec2.len());
    let s1 = vec1.iter().sum::<f32>();
    let s2 = vec2.iter().sum::<f32>();

    let mut dot_product = 0.0;
    let mut vec1_norm = 0.0;
    let mut vec2_norm = 0.0;
    for (v1, v2) in vec1.iter().zip(vec2.iter()) {
        let v1 = apply_sub_raw_mean(*v1, vec1_mean);
        let v2 = apply_sub_raw_mean(*v2, vec2_mean);
        let v1 = v1 - s1 / vec1.len() as f32;
        let v2 = v2 - s2 / vec2.len() as f32;
        dot_product += v1 * v2;
        vec1_norm += v1.powi(2);
        vec2_norm += v2.powi(2);
    }
    dot_product / (vec1_norm.sqrt() * vec2_norm.sqrt())
}

fn get_cosine_raw(raw1: CsrRow<f32>, raw2: CsrRow<f32>) -> f32 {
    let mut dot_product = 0.0;
    let mut vec1_norm = 0.0;
    let mut vec2_norm = 0.0;
    
    for col in raw1.col_indices().iter().chain(raw2.col_indices().iter()).collect::<HashSet<_>>() {
        let v1 = raw1.get_entry(*col);
        let v2 = raw2.get_entry(*col);
        
        let v1 = if let Some(v1) = v1 {
            v1.into_value().powi(2)
        } else {
            0.0
        };

        let v2 = if let Some(v2) = v2 {
            v2.into_value().powi(2)
        } else {
            0.0
        };
        
        dot_product += v1 * v2;
        vec1_norm += v1.powi(2);
        vec2_norm += v2.powi(2);
    }

    dot_product / (vec1_norm.sqrt() * vec2_norm.sqrt())
}

#[inline]
fn apply_sub_raw_mean(value: f32, mean: f32) -> f32 {
    if value != 0.0 {
        value - mean
    } else {
        0.0
    }
}

/// Compute the similarity between two users based on their ratings
/// We use the cosine similarity between the two vectors of ratings
/// As the vectors doen't have the same length, we use the union of the two sets of movies
/// and fill the missing ratings with 0
pub fn get_similarity(set1: HashMap<MID, u8>, set2: HashMap<MID, u8>) -> f32 {
    // len vec of ratings = |set1 union set2|
    let movies_id: HashSet<MID> = HashSet::from_iter(set1.keys().chain(set2.keys()).cloned());
    let mut vec1_ratings = Vec::with_capacity(movies_id.len());
    let mut vec2_ratings = Vec::with_capacity(movies_id.len());
    let (mut s1, mut s2) = (0.0, 0.0);
    for movie_id in movies_id {
        let v1 = *set1.get(&movie_id).unwrap_or(&0) as f32;
        let v2 = *set2.get(&movie_id).unwrap_or(&0) as f32;
        s1 += v1;
        s2 += v2;
        vec1_ratings.push(v1);
        vec2_ratings.push(v2);
    }

    get_cosine(
        vec1_ratings.clone(),
        vec2_ratings.clone(),
        s1 / vec1_ratings.len() as f32,
        s2 / vec2_ratings.len() as f32,
    )
}

pub fn compute_matrix(similar_users: Vec<(UID, HashMap<MID, u8>)>, me: HashMap<MID, u8>) -> CooMatrix<f32> {
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

pub fn computer_similarities(matrix: CsrMatrix<f32>) -> Vec<f32> {
    let mut similarities = Vec::with_capacity(matrix.nrows());
    let user = matrix.row(matrix.nrows() - 1);
    for row in matrix.row_iter() {
        similarities.push(get_cosine_raw(
            user.clone(),
            row,
        ));
    }
    similarities
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
    /// Simularities mult Matrix
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
    recos: &BTreeMap<MID, HashMap<UID, u8>>,
    users: &BTreeMap<UID, HashMap<MID, u8>>,
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