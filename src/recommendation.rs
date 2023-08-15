use crate::prelude::*;

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
    // Get the most similar user to the query user
    let mut most_similar_user = 0;
    let mut max_similar_user = 0.0;
    for (user, movies) in similar_users {
        let similarity = get_similarity(query.ratings.clone(), movies);
        if similarity > max_similar_user && user != query.user_id {
            max_similar_user = similarity;
            most_similar_user = user;
        }
    }
    println!("most_similar_user: {} {}", most_similar_user, max_similar_user);
    let view_movies_most_similar_user = users
        .get(&most_similar_user);
    
    if view_movies_most_similar_user.is_none() {
        return Vec::new();
    }
    let view_movies_most_similar_user = view_movies_most_similar_user.unwrap();   
    // Get the movie with the highest rating from the most similar user that the query user hasn't seen
    let mut top_movies = Vec::new();
    for movie in view_movies_most_similar_user {
        if movie.1 > &9 && !query.ratings.contains_key(movie.0) {
            top_movies.push((*movie.0, *movie.1));
        }
    }
    top_movies
}
