# Lsor

Lsor is an opinionated kind-of-ORM-but-not-really that allows you to write SQL
statements using Rust (through an unholy marriage between Async-GraphQL, PRQL
and SQLX).

```rs
#[derive(Filter, Row, Sort)]
#[lsor(table = "users")]
pub struct User {
    #[lsor(pk)]
    pub id: Uuid,
    pub email: String,
}
```

Now we can query this over GraphQL, and `lsor` will have auto-magically generated
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

Is this a good idea? I don't know. But it's fun to write, and it's fun to use.