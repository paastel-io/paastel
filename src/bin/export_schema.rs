use async_graphql::{EmptySubscription, Schema};

use paastel::graphql::{mutation::MutationRoot, query::QueryRoot};

fn main() {
    let schema =
        Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();
    std::fs::write("schema.graphql", schema.sdl()).unwrap();
    println!("Schema salvo em schema.graphql");
}
