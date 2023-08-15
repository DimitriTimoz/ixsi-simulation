pub mod movies;
pub mod prelude;
pub mod recommendation;

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
        let query = RecoQuery::from(
            user.ratings
                .iter()
                .take((user.ratings.len() as f32 * 0.8) as usize)
                .cloned()
                .collect::<Vec<Movie>>(),
        );
        let recos = get_recommendations(&recos, &users, &query).await;
        for (movie, rating) in &recos {
            for viewed_movie in &user.ratings {
                if *movie == viewed_movie.id {
                    println!(
                        "movie expet rating: {}, rated: {:?}",
                        rating, viewed_movie.rating
                    );
                }
            }
        }
    }
}
