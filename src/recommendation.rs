use crate::prelude::*;



pub async fn get_recommendations(recos: &BTreeMap<Movie, HashSet<usize>>, query: &RecoQuery) -> Movie {
    // Get all similar users
    let mut similar_users = HashSet::new();
    for movie in &query.ratings {
        if let Some(users) = recos.get(movie) {
            similar_users.extend(users);
        }
    }
    
    
    Movie::new(1, 10)
}