use std::collections::HashSet;

use log::{error, info, LevelFilter};
use meilisearch_sdk::Client;
use meilisearch_sdk::documents::DocumentsQuery;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::SearchResults;
use pretty_env_logger::env_logger::Builder;
use tokio::task;

use common::{API_KEY, CNT_MOVIES, IDX_MOVIE, IDX_PERSON, IDX_PRINCIPAL, IDX_SEARCH_DOC, SERVER};
use common::models::{Movie, Person, Principal, print_res, SearchDoc};

#[tokio::main]
async fn main() {
    Builder::new().filter_level(LevelFilter::Info).init();

    // change these 2 values to force higher load and the 408 error
    let cnt_tasks = 4;
    let limit = 100;         // page_size for reading movie documents

    let movies_per_task = CNT_MOVIES / cnt_tasks;

    info!("cnt_tasks {cnt_tasks}, total_movies {CNT_MOVIES}, movies_per_task {movies_per_task}, limit {limit}");
    let mut handles = vec![];
    for i in 0..cnt_tasks {
        let client = Client::new(SERVER, Some(API_KEY));

        let mut offset = movies_per_task * i;

        let h = task::spawn(async move {
            info!("new thread starts at offset: {:?}, limit {}", offset, limit);

            let index = client
                .index(IDX_MOVIE);

            let mut movies_processed = 0;
            let mut movies_found = true;
            while movies_found && movies_processed < movies_per_task {
                let movies = read_movies_paginated(limit, offset, &index).await;

                info!("found {} movies ", movies.len());
                let mut search_docs = read_entities_and_combine_to_search_doc(&client, &movies).await;

                let idx_search_docs = client.index(IDX_SEARCH_DOC);
                let res = idx_search_docs.add_documents(&search_docs, Some("id")).await;
                print_res(res, "error inserting search_docs.".to_string(), format!("inserted {} search_docs", search_docs.len()));
                search_docs.clear();

                offset += limit;
                movies_processed += movies.len();
                movies_found = !movies.is_empty();

                info!("movies_processed {movies_processed}, movies.len() {},  total_movies {CNT_MOVIES}, movies_per_task {movies_per_task}, limit {limit}",movies.len());
            }

            movies_processed
        });
        handles.push(h);
    }

    for h in handles {
        let h = h.await;
        match h {
            Ok(cnt) => info!("task finished. {} movies processed", cnt),
            Err(e) => error!("task crashed and returned an error {}",e),
        }
    }
}

async fn read_entities_and_combine_to_search_doc(client: &Client, movies: &Vec<Movie>) -> Vec<SearchDoc> {
    let mut search_docs = vec![];

    for movie in movies {
        let principals = find_principals_for_movie(movie.tconst.clone(), &client).await;
        let mut nconsts = HashSet::new();

        for p in &principals {
            nconsts.insert(p.nconst.clone());
        }
        let mut persons = find_persons_by_nconst(&nconsts, &client).await;
        info!(" movie tconst: {}  -> principals found: {} found  {} persons using these  principals nconsts: '{:?}' ", movie.tconst,persons.len(), principals.len(), nconsts);
        let names: Vec<String> = persons.drain(..).map(|p| p.primary_name).collect();
        let doc = get_search_doc(movie, names);
        search_docs.push(doc);
    };
    search_docs
}

async fn read_movies_paginated(limit: usize, offset: usize, index: &Index) -> Vec<Movie> {
    let res = DocumentsQuery::new(&index)
        .with_limit(limit)
        .with_offset(offset)
        .execute::<Movie>()
        .await;

    let movies_result = match res {
        Ok(docs) => {
            docs
        }
        Err(e) => {
            error!("error reading movies {}", e);
            panic!("should not happen. error reading movies.");
        }
    };

    movies_result.results
}

async fn find_principals_for_movie(tconst: String, client: &Client) -> Vec<Principal> {
    let filter = format!("(\"tconst\" = \"{tconst}\")");
    let principals: SearchResults<Principal> = client
        .index(IDX_PRINCIPAL)
        .search()
        .with_filter(&filter)
        .execute()
        .await
        .unwrap();

    let mut p = principals.hits;
    p.drain(..).map(|pp| pp.result).collect::<Vec<Principal>>()
}

async fn find_persons_by_nconst(nconsts: &HashSet<String>, client: &Client) -> Vec<Person> {
    let filter = nconsts.iter().map(|nconst| format!("\"nconst\" = \"{nconst}\""))
        .collect::<Vec<String>>()
        .join(" OR ");
    let filter = format!("( {} ) ", filter);
    let res = client
        .index(IDX_PERSON)
        .search()
        .with_filter(&filter)
        .execute()
        .await;

    let mut persons = match res {
        Ok(docs) => {
            docs.hits
        }
        Err(e) => {
            info!("error reading persons {}", e);
            panic!("should not happen. error reading persons.");
        }
    };

    persons.drain(..).map(|pp| pp.result).collect::<Vec<Person>>()
}

fn get_search_doc(movie: &Movie, names: Vec<String>) -> SearchDoc {
    SearchDoc {
        id: movie.tconst.clone(),
        tconst: movie.tconst.clone(),
        title: movie.title.clone(),
        genres: movie.genres.clone(),
        names,
    }
}
