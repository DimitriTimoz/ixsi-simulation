pub mod movies;
pub mod recommendation;
pub mod prelude;

use crate::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (recos, picked_users) = movies::get_recos_and_random_users();
    let query = movies::RecoQuery::from(vec![movies::Movie::new(1, 10), movies::Movie::new(2, 10)]);
    let recos = get_recommendations(&recos, &query).await;
}
