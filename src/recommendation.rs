use crate::prelude::*;


fn get_cosine(vec1: Vec<f32>, vec2: Vec<f32>) -> f32 {
    assert!(vec1.len() == vec2.len());
    let s1 = vec1.iter().sum::<f32>();
    let s2 = vec2.iter().sum::<f32>();

    let mut dot_product = 0.0;
    let mut vec1_norm = 0.0;
    let mut vec2_norm = 0.0;
    for (v1, v2) in vec1.iter().zip(vec2.iter()) {
        let v1 = v1 - s1 / vec1.len() as f32;
        let v2 = v2 - s2 / vec2.len() as f32;
        dot_product += v1 * v2;
        vec1_norm += v1.powi(2);
        vec2_norm += v2.powi(2);
    }
    dot_product / (vec1_norm.sqrt() * vec2_norm.sqrt())
}

pub fn get_similarity(vec1: Vec<Movie>, vec2: Vec<Movie>) -> () {
    
}


pub async fn get_recommendations(recos: &BTreeMap<Movie, HashSet<usize>>, users: &BTreeMap<usize, HashSet<Movie>>, query: &RecoQuery) -> Vec<Movie> {
    // Get all similar users
    let mut similar_users: HashMap<usize, Vec<Movie>> = HashMap::new();
    for movie in &query.ratings {
        if let Some(users) = recos.get(movie) {
            for user in users {
                if query.ratings.contains(movie) {
                    similar_users.entry(*user).or_insert_with(Vec::new).push(movie.clone());
                }
            }
        }
    }
    // Get the most similar user to the query user
    let mut most_similar_user = 0;
    let mut max_similar_user = 0;
    for (user, movies) in similar_users {
        if movies.len() > max_similar_user && user != query.user_id{
            max_similar_user = movies.len();
            most_similar_user = user;
        }
    }
    println!("most_similar_user: {}", most_similar_user);
    let view_movies_most_similar_user = users.get(&most_similar_user).unwrap_or(&HashSet::new()).clone();
    // Get the movie with the highest rating from the most similar user that the query user hasn't seen
    let mut top_movies = Vec::new();
    for movie in view_movies_most_similar_user {
        if movie.rating > 8 && !query.ratings.contains(&movie) {
            top_movies.push(movie.clone());
        }
    }
    top_movies
}