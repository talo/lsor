# Lsor

Lsor is an opinionated kind-of-ORM-but-not-really that allows you to write SQL
statements using Rust (through an unholy marriage between [Async-GraphQL](https://github.com/async-graphql/async-graphql), [PRQL](https://github.com/prql/prql)
and [SQLX](https://github.com/launchbadge/sqlx)).

```rs
#[derive(Filter, Row, Sort)]
#[lsor(table = "users")]
pub struct User {
    #[lsor(primary_key)]
    pub id: Uuid,
    pub email: String,
}
```

Now we can query this over GraphQL, and Lsor will have auto-magically generated
the necessary `UserFilter` and `UserSort` types.

```graphql
query Q1 {
    users(filter: { id: { lt: '88888888-8888-8888-8888-888888888888' } }, sort: { email: desc }) {
        edges {
            node {
                id
                email
            }
        }
    }
}
```

The `lsor` DSL also allows us to use the `UserFilter`, `UserSort`, and `User`
types to write our SQL statements.

```rs
let mut driver = Driver::new();
from(User::table())
    .filter(UserFilter::Id(UuidFilter::Lt(Uuid::new_v4())))
    .sort(UserSort::Id(StringSort::Desc))
    .push_to_driver(&mut driver);
let user: User = driver.fetch_one(&pool).await?;
```

Is this a good idea? I don't know. But it's fun to write, and it's nice to use.

## Example

This is a slightly more complete example. At some point we will probably write some tutorials, but that point is not now.

### Data access layer

First, we define a struct to represent our auth token:

```rs
#[derive(Clone, Debug, Filter, Row, SimpleObject, Sort)]
#[graphql(rename_fields = "snake_case")]
#[lsor(table = "tokens")]
pub struct Token {
    #[lsor(primary_key)]
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,

    #[graphql(skip)]
    pub user_id: Uuid,
}
```

Nothing super crazy. It will be uniquely identified by its `id`, and has a `user_id` field, which is a foreign key to the `users` table. Lsor is meant to be used alongside `async-graphql` so we also derive `SimpleObject`.

Next, we define a struct to represent our user:

```rs
#[derive(Clone, Debug, Filter, Row, SimpleObject, Sort)]
#[graphql(complex, rename_fields = "snake_case")]
#[lsor(table = "users")]
pub struct User {
    #[lsor(primary_key)]
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
```

Again, nothing crazy. Arguably even less crazy than the already tame `Token`. But, now are ready to define our first GraphQL query:

```rs
#[ComplexObject(rename_fields = "snake_case")]
impl User {
    #[allow(clippy::too_many_arguments)]
    async fn tokens(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
        filter: Option<TokenFilter>,
        sort_by: Option<TokenSort>,
    ) -> async_graphql::Result<Connection<String, Token, TotalCount, EmptyFields>> {
        // Get the SQLX pool (assuming that it has been injected during the
        // GraphQL schema initialisation)
        let pool = ctx.data::<Pool<Postgres>>().unwrap();

        // Create a filter that will filter for tokens that belong to this
        // user
        let filter_by_user_id = TokenFilter::UserId(UuidFilter::Eq(self.id));

        // Default to sorting by `created_at` in descending order (i.e. the
        // most recent tokens first)
        let sort = sort.unwrap_or(TokenSort::CreatedAt(DateTimeSort::Desc));
        let cursor = sort.cursor();

        lsor::load_page(
            &self.pool,
            if_then_else(
                filter.is_some(),
                || and(filter.unwrap(), &filter_by_user_id),
                || &filter_by_user_id,
            ),
            sort,
            Pagination {
                cursor,
                after,
                before,
                first: first.unwrap_or(10).clamp(1, 100) as usize,
                last: last.unwrap_or(10).clamp(1, 100) as usize,
            },
        )
        .await

        Ok(conn)
    }
}
```

A bit crazier, but upon closer inspection it is mostly boilerplate. The actual data access logic is stupidly simple.

This is all you need for a fully GraphQL specification-compliant connection resolver. The `lsor::load_page` function will take care of the heavy lifting for you, and will return a `Connection` object that you can return to the client.

If you want to take a look under-the-hood at how `lsor::load_page` works, then head on over to the `lsor::page` module in this crate. It is nothing complicated, but it is actually a pretty good example of what's possible with `lsor`.

### GraphQL query

The auto-magical creation of the `TokenFilter` and `TokenSort` types means that we can now write GraphQL queries that looks like this:

```graphql
query Q1 {
    # assuming `me` returns the `User` type
    me {
        tokens(filter: { created_at: { gt: "2021-01-01T00:00:00Z" } }, sort: { updated_at: asc }) {
            edges {
                node {
                    id
                    createdAt
                    updatedAt
                    deletedAt
                }
            }
        }
    }
}
```

This query will return a list of tokens that belong to the user that is currently logged in, and that were created after the 1st of January 2021. The tokens will be sorted by their `updated_at` field in ascending order. Lsor has taken care of all the necessary PRQL/SQL code to make this work.

Pretty neat, eh? 

### Setup

One last thing to make sure the picture is mostly complete: what does `main.rs` look like? We use `axum` so it looks something like this:

```rs
#[tokio::main]
async fn main() {
    let pool = Pool::<Postgres>::connect("postgres://localhost:5432").await?,

    let schema = Schema::build(
        Query,
        Mutation,
        Subscription,
    )
    .data(pool)
    .finish();

    let app = Router::new()
        .route("/", routing::get(playground).post(queries_and_mutations))
        .route("/graphiql", routing::get(graphiql))
        .layer(Extension(schema))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024));

    let listener = TcpListener::bind("0.0.0.0:4443".parse()?).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn playground() -> impl IntoResponse {
    Html(playground_source(
        GraphQLPlaygroundConfig::new("/"),
    ))
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/")
            .finish(),
    )
}

async fn queries_and_mutations(
    schema: Extension<Schema<Query, Mutation, Subscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
```

### Why PRQL

Lsor takes your Rust expressions and uses them to emit PRQL. This PRQL is then compiled into SQL (specificially for Postgres) using SQLX.

Why? Because going directly into SQL is both incredibly annoying to do, and also has soundness and completeness issues due to some fundamental differences between SQL and Rust. Luckily for us, the good folks over at PRQL have done all the hard work.