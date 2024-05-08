pub mod movies;
pub mod prelude;
pub mod recommendation;

use nalgebra_sparse::CsrMatrix;
use recommendation::compute_norms;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (matrix, users, means) = movies::get_matrix_and_ratings();
    println!("len users: {}", users.len());
    let matrix = CsrMatrix::from(&matrix);
    
    println!("normalizing matrix");
    //normalize_matrix(&mut matrix);

    println!("computing norms");
    let norms = compute_norms(&matrix);
    let (mut dumb_cost, mut algo_cost) = (0.0, 0.0);

    for (i, user) in users.iter().enumerate() {
        let reco = recommendation::get_recommendations(&matrix, &norms, i).await;

        // Check if the recommendations are valid
        let (user_id, user_ratings) = user;
        for (movie, rating) in reco {
            let rating = rating * 5.0;
            let real_rating = user_ratings.get(&movie).unwrap_or(&0.0);
            if real_rating > &0.0 {
                //println!("Movie {} was rated {} by user {} and the recommendation is {}, difference: {} {} ", movie, real_rating, user_id, real_rating, real_rating - rating, real_rating - means[movie]);
                algo_cost += (real_rating - rating).powi(2);
                dumb_cost += (real_rating - means[movie]).powi(2);
            } 
        }
        println!("User {} algo cost: {}, dumb cost: {}", user_id, algo_cost, dumb_cost);
    }
}
