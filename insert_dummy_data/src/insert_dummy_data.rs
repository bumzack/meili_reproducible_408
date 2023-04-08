use log::info;
use meilisearch_sdk::client::*;

use common::{IDX_MOVIE, IDX_PERSON, IDX_PRINCIPAL};
use common::models::{Movie, print_res};
use common::models::Person;
use common::models::Principal;

pub async fn insert_dummy_data_movies(cnt_movies: usize, batch_size: usize, client: &Client) {
    let idx_movies = client.index(IDX_MOVIE);

    let mut movies = vec![];
    for i in 1..=cnt_movies {
        let movie = get_movie(i);
        movies.push(movie);

        if movies.len() >= batch_size {
            info!("inserting documents into index.  movies {} / {}.    movies.len(): {}", i, cnt_movies, movies.len());
            let res = idx_movies.add_documents(&movies, Some("id")).await;
            print_res(res, "error inserting movies.".to_string(), format!("cnt movies {}", movies.len()));
            movies.clear();
        }
    }
}

pub async fn insert_dummy_data_person(cnt_persons: usize, batch_size: usize, client: &Client) {
    info!("insert_dummy_data_person  {}", cnt_persons);
    let cnt_persons = cnt_persons / 2;
    let batch_size = batch_size / 2;

    let idx_person = client.index(IDX_PERSON);

    let mut persons = vec![];
    let mut idx = 1;
    for i in 1..=cnt_persons {
        let p = get_person_arnold(idx);
        idx += 1;
        persons.push(p);
        let p = get_person_linda(idx);
        idx += 1;
        persons.push(p);

        if persons.len() >= batch_size {
            info!("inserting documents into index.  persons {} / {}.    persons.len(): {}", i, cnt_persons, persons.len());
            let res = idx_person.add_documents(&persons, Some("id")).await;
            print_res(res, "error inserting persons.".to_string(), format!("cnt persons {}", persons.len()));
            persons.clear();
        }
    }
}

pub async fn insert_dummy_data_principal(cnt_movies: usize, batch_size: usize, client: &Client) {
    info!("insert_dummy_data_principal  cnt_movies {}. insert 6 persons <-> movies relations. ", cnt_movies);
    let idx_principal = client.index(IDX_PRINCIPAL);

    let mut principals = vec![];
    let mut idx = 1;

    // for each movie insert 6 persons. 3x arnold, 3x linda
    for tconst in 1..=cnt_movies {
        for p_movie in 0..3 {
            let p = get_principal_arnold(idx, tconst, p_movie);
            idx += 1;
            principals.push(p);

            let p = get_principal_linda(idx, tconst, p_movie);
            idx += 1;
            principals.push(p);
        }

        if principals.len() >= batch_size {
            info!("inserting documents into index.  principals for movies {} / {}.    principals.len(): {}", tconst, cnt_movies, principals.len());
            let res = idx_principal.add_documents(&principals, Some("id")).await;
            print_res(res, "error inserting principals.".to_string(), format!("cnt principals {}", principals.len()));
            principals.clear();
        }
    }
}

fn get_movie(i: usize) -> Movie {
    let movie = Movie {
        id: i.to_string(),
        tconst: i.to_string(),
        title: "Terminator".to_string(),
        genres: vec!["action".to_string(), "comedy".to_string()],
    };
    movie
}

fn get_person_linda(idx: i32) -> Person {
    Person {
        id: idx.to_string(),
        nconst: idx.to_string(),
        primary_name: "Linda Hamilton".to_string(),
        birth_year: Some(1956),
        death_year: None,
        primary_profession: vec!["actress".to_string()],
        known_for_titles: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
    }
}

fn get_person_arnold(idx: i32) -> Person {
    Person {
        id: idx.to_string(),
        nconst: idx.to_string(),
        primary_name: "Arnold Schwarzenegger".to_string(),
        birth_year: Some(1947),
        death_year: None,
        primary_profession: vec!["actor".to_string()],
        known_for_titles: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
    }
}

fn get_principal_linda(idx: i32, tconst: usize, p_movie: usize) -> Principal {
    Principal {
        id: idx.to_string(),
        tconst: tconst.to_string(),
        nconst: "2".to_string(),
        category: "actress".to_string(),
        ordering: p_movie,
        characters: vec!["Sarah Connor".to_string()],
    }
}

fn get_principal_arnold(idx: i32, tconst: usize, p_movie: usize) -> Principal {
    Principal {
        id: idx.to_string(),
        tconst: tconst.to_string(),
        nconst: "1".to_string(),
        category: "actor".to_string(),
        ordering: p_movie,
        characters: vec!["Terminator".to_string()],
    }
}
