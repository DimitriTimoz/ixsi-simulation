pub mod movies;
pub mod prelude;
pub mod recommendation;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (recos, users, counts) = movies::get_matrix_and_ratings();
    println!("counts: {:?}", counts);
    println!("len users: {}", users.len());
    
}
