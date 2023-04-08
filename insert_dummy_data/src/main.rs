use log::{info, LevelFilter};
use meilisearch_sdk::client::*;
use meilisearch_sdk::task_info::TaskInfo;
use pretty_env_logger::env_logger::Builder;
use tokio::io;

use common::{API_KEY, BATCH_SIZE, CNT_MOVIES, CNT_PERSONS, IDX_PERSON, IDX_PRINCIPAL, SERVER};

use crate::insert_dummy_data::{insert_dummy_data_movies, insert_dummy_data_person, insert_dummy_data_principal};

mod insert_dummy_data;


#[tokio::main]
async fn main() -> io::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    // from https://github.com/meilisearch/meilisearch-rust
    // Create a client (without sending any request so that can't fail)
    let client = Client::new(SERVER, Some(API_KEY));

    let filterable_attributes_person = [
        "nconst",
    ];

    let t: TaskInfo = client
        .index(IDX_PERSON)
        .set_filterable_attributes(&filterable_attributes_person)
        .await
        .unwrap();
    info!("taskinfo person filterable attributes {:?}", t);

    let filterable_attributes_principial = [
        "tconst",
    ];

    let t: TaskInfo = client
        .index(IDX_PRINCIPAL)
        .set_filterable_attributes(&filterable_attributes_principial)
        .await
        .unwrap();

    info!("taskinfo principals filterable attributes {:?}", t);

    info!("inserting movie");
    let _ = insert_dummy_data_movies(CNT_MOVIES, BATCH_SIZE, &client).await;

    info!("inserting persons");
    let _ = insert_dummy_data_person(CNT_PERSONS, BATCH_SIZE, &client).await;

    info!("inserting principals");
    let _ = insert_dummy_data_principal(CNT_MOVIES, BATCH_SIZE / 10, &client).await;

    info!("done");
    Ok(())
}
