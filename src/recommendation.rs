use crate::prelude::*;

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