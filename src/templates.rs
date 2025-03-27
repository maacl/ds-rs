use maud::html;

pub fn dbmon_app(dbs: Vec<crate::DatabaseWithData>) -> maud::Markup {
    html! {
        table id="app" class="table table-xs w-full" {
            tbody {
                @for db in dbs {
                    (dbmon_database(db))
                }
            }
        }
    }
}

fn dbmon_database(db: crate::DatabaseWithData) -> maud::Markup {
    html! {
        tr {
            td class="dbname" { (db.name) }
            td class=(db.counter_classes) {
                span class=(db.counter_classes) {
                    (db.query_count)
                }
            }
            @for query in &db.top5_queries {
                td class="text-xs font-mono" {
                    div class="tooltip" data-tip=(query.query) {
                        (query.elapsed) " ms"
                    }
                }
            }
        }
    }
}
