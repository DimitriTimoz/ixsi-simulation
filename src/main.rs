pub mod movies;
pub mod recommendation;
pub mod prelude;

use crate::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (recos, users, picked_users, counts) = movies::get_recos_and_random_users();
    println!("counts: {:?}", counts);
    println!("len recos: {}", recos.len());
    println!("len users: {}", users.len());
    println!("len picked_users: {}", picked_users.len());
    for user in &picked_users {
        // Query contains 90% of the user's ratings
        let query = RecoQuery::from(user.ratings.iter().take((user.ratings.len() as f32 * 0.01) as usize).cloned().collect::<Vec<Movie>>());
        let recos = get_recommendations(&recos, &users, &query).await;
        for movie in &recos {
            if let Some(viewed) = user.ratings.get(movie) {
                println!("Recommendation: {} is in user: {:?}", movie.id, user.user_id);
                //Compare to the recommendation
                println!("Rating: {} expected: {}", viewed.rating, movie.rating);
            } 
        }
        
    }

}
