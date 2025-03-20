mod templates;

use {
    datastar::{prelude::MergeFragments, DatastarEvent, Sse},
    rand::Rng,
    rocket::{
        futures::Stream,
        get, launch,
        response::{content::RawHtml, stream::stream},
        routes,
    },
    serde::Serialize,
    std::{thread::sleep, time::Duration},
    templates::page_dbmon,
};

#[derive(Clone, Debug, Serialize)]
struct DbmonQuery {
    elapsed: u128,
    query: String,
}

impl DbmonQuery {
    fn random() -> Self {
        let mut rng = rand::rng();
        let elapsed = Duration::from_millis((rng.random::<f64>() * 15.0) as u64).as_millis();
        let query = if rng.random::<f32>() < 0.2 {
            "<IDLE> in transaction".to_string()
        } else if rng.random::<f32>() < 0.1 {
            "vacuum".to_string()
        } else {
            "SELECT blah from something".to_string()
        };

        DbmonQuery { elapsed, query }
    }
}

#[derive(Clone, Debug, Serialize)]
struct DbmonDatabase {
    name: String,
    queries: Vec<DbmonQuery>,
}

impl DbmonDatabase {
    fn new(name: String) -> Self {
        let mut db = DbmonDatabase {
            name,
            queries: Vec::new(),
        };
        db.update();
        db
    }

    fn update(&mut self) {
        let mut rng = rand::rng();
        let count = rng.random_range(1..=15);
        self.queries = (0..count).map(|_| DbmonQuery::random()).collect();
    }

    fn top5_queries(&self) -> Vec<DbmonQuery> {
        let mut queries = self.queries.clone();
        queries.sort_by(|a, b| a.elapsed.cmp(&b.elapsed));
        queries.truncate(5);
        while queries.len() < 5 {
            queries.push(DbmonQuery {
                elapsed: 0,
                query: String::new(),
            });
        }
        queries
    }
}

#[derive(Clone, Serialize)]
struct DbmonDatabases {
    databases: Vec<DbmonDatabase>,
}

impl DbmonDatabases {
    fn new(n: usize) -> Self {
        let mut databases = Vec::with_capacity(n * 2);
        for i in 0..n {
            databases.push(DbmonDatabase::new(format!("cluster{}", i)));
            databases.push(DbmonDatabase::new(format!("cluster{}follower", i)));
        }
        DbmonDatabases { databases }
    }

    fn random_update(&mut self, rate: f64) {
        let mut rng = rand::rng();
        for db in &mut self.databases {
            if rng.random::<f64>() < rate {
                db.update();
            }
        }
    }
}

fn dbmon_counter_classes(count: usize) -> String {
    if count >= 15 {
        "bg-red-200".to_string()
    } else if count >= 10 {
        "bg-orange-200".to_string()
    } else {
        "bg-green-200".to_string()
    }
}

const INDEX: &str = include_str!("../index.html");

#[get("/")]
fn index() -> RawHtml<String> {
    RawHtml(INDEX.into())
}

// Helper struct to hold precomputed data for the template
#[derive(Serialize, Debug)]
struct DatabaseWithData {
    name: String,
    query_count: usize,
    counter_classes: String,
    top5_queries: Vec<DbmonQuery>,
}

#[get("/updates")]
fn updates() -> Sse<impl Stream<Item = DatastarEvent>> {
    let mut dbs = DbmonDatabases::new(6);

    Sse(stream! {
        loop {
            dbs.random_update(0.2);
            let dbs_with_data: Vec<DatabaseWithData> = dbs
                  .databases
                  .iter()
                  .map(|db| {
                let counter_classes = dbmon_counter_classes(db.queries.len());
                let top5_queries = db.top5_queries();
                DatabaseWithData {
                      name: db.name.clone(),
                      query_count: db.queries.len(),
                      counter_classes,
                      top5_queries: top5_queries.clone(),
                }
                })
                .collect();
            let content = page_dbmon(dbs_with_data);

            yield MergeFragments::new(content).into();
            sleep(Duration::from_millis(100));
        }
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, updates])
}
