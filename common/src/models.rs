use log::{error, info};
use meilisearch_sdk::errors::Error;
use meilisearch_sdk::task_info::TaskInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Movie {
    pub id: String,
    pub tconst: String,
    pub title: String,
    pub genres: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Principal {
    pub id: String,
    pub tconst: String,
    pub nconst: String,
    pub category: String,
    pub ordering: usize,
    pub characters: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: String,
    pub nconst: String,
    pub primary_name: String,
    pub birth_year: Option<u32>,
    pub death_year: Option<u32>,
    pub primary_profession: Vec<String>,
    pub known_for_titles: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchDoc {
    pub id: String,
    pub tconst: String,
    pub title: String,
    pub genres: Vec<String>,
    pub names: Vec<String>,
}


pub fn print_res(res: Result<TaskInfo, Error>, msg: String, success_msg: String) {
    match res {
        Ok(res) => {
            info!("success {},           res {:?}",success_msg ,res);
        }
        Err(e) => {
            error!("{} {}", msg, e);
        }
    }
}
