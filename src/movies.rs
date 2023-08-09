use crate::prelude::*;
use rand::{self, Rng};

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Movie {
    pub id: usize,
    pub rating: u8,
}

impl Movie {

    pub fn new(id: usize, rating: u8) -> Self {
        Self { id, rating }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct UserRatings {
    pub user_id: usize,
    pub ratings: HashSet<Movie>,
}

#[derive(Clone, Debug)]
pub struct RecoQuery {
    pub user_id: usize,
    pub ratings: HashSet<Movie>,
}

impl From<UserRatings> for RecoQuery {
    fn from(user_ratings: UserRatings) -> Self {
        Self { ratings: user_ratings.ratings, user_id: user_ratings.user_id }
    }
}

impl From<Vec<Movie>> for RecoQuery {
    fn from(ratings: Vec<Movie>) -> Self {
        Self {
            ratings: ratings.into_iter().collect(),
            user_id: 0,
        }
    }
}


pub fn get_recos_and_random_users() -> (BTreeMap<Movie, HashSet<usize>>, BTreeMap<usize, HashSet<Movie>>,Vec<UserRatings>, [usize; 10]) {
    let mut ratings_count = [0; 10];
    // Read the lines
    let file = read_to_string("ratings.csv").expect("Failed to open file: ratings_small.csv seems doesn't exist");
    let mut reviews = BTreeMap::new();
    
    let n_users = 300;
    let mut rng = rand::thread_rng();
    // Ids of users_picked to be selected pick 100 random users
    let users_picked = (0..n_users).map(|_| rng.gen_range(0..200_000)).collect::<HashSet<usize>>();
    let mut users_picked: HashMap<usize, HashSet<Movie>> = HashMap::from_iter(users_picked.into_iter().map(|user_id| (user_id, HashSet::new())));

    let mut users = BTreeMap::new();

    for line in file.lines().skip(1)  {
        let mut line = line.split(',');
        let user_id = line.next().unwrap().parse::<usize>().unwrap();
        let movie_id = line.next().unwrap().parse::<usize>().unwrap();
        let rating = line.next().unwrap().parse::<f32>().unwrap();
        let rating = (rating*2.0).round() as u8;
        ratings_count[rating as usize - 1] += 1;
        let movie = Movie::new(movie_id, rating);
        // Add the movie to the user's list of movies if is selected
        if let Some(view_movies) = users_picked.get_mut(&user_id) {
            view_movies.insert(movie.clone());
        } else {
            let user_ratings = reviews.entry(movie.clone()).or_insert(HashSet::new());
            user_ratings.insert(user_id);
            let user_ratings = users.entry(user_id).or_insert(HashSet::new());
            user_ratings.insert(movie);
    
        }

    }

    (reviews, users, users_picked.into_iter().map(|(user_id, ratings)| UserRatings { user_id, ratings }).collect(), ratings_count)

}
