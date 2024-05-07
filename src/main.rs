pub mod movies;
pub mod prelude;
pub mod recommendation;

use nalgebra_sparse::CsrMatrix;
use recommendation::compute_norms;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (matrix, users) = movies::get_matrix_and_ratings();
    println!("len users: {}", users.len());
    let matrix = CsrMatrix::from(&matrix);
    
    println!("normalizing matrix");
    //normalize_matrix(&mut matrix);

    println!("computing norms");
    let norms = compute_norms(&matrix);
    

    for (i, user) in users.iter().enumerate() {
        let reco = recommendation::get_recommendations(&matrix, &norms, i).await;
    }
    
}
