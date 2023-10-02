# Making Changes to the Database

This document describes how to make changes to the database schema.

> **⚠ Warning:**
> Any changes to the database schema ***MUST*** be backwards compatible. This allows us to have zero downtime deployments. See [Migration Backwards Compatibility](#migration-backwards-compatibility) for more information.

## Prerequisites

You will need the CLI tool `sea-orm-cli` to create migrations. You can install it with the following command:

```bash
cargo install sea-orm-cli
```

If you prefer to use `binstall`, you can run the following command instead:

```bash
cargo binstall sea-orm-cli
```

Additionaly, you will have to set up the environment variables for the database. You can do this by creating a `.env` file in the root of the repository. The contents of the file should be as follows:

```bash
DATABASE_URL=postgres://<username>:<password>@<host>:<port>/<database>
```

## Creating a Migration

To modify the database schema, you must create a migration. A migration is a file that contains the what the database should look like after the migration is applied. It also tells the database how to migrate from the previous version of the database to the new version.

Every change to a table or column must be done in a migration. This includes creating, modifying, and deleting tables and columns. Every change should be done in its own migration. This makes it easier to roll back changes if necessary.

To create a migration, run the following command:

```bash
sea-orm-cli migrate generate <migration-name> -d ./crates/migrations/
```

This will create a new file in the [`/crates/migrations/src/migrations`](/crates/migrations/src/migrations) directory. The file name will be in the format `<timestamp>_<number>_<migration_name>.rs`. The `migration_name` is the name you specified in the command above.

## Modifying the Migration

The migration file will contain two functions: `up` and `down`. The `up` function is called when the migration is applied. The `down` function is called when the migration is rolled back.

### How to Modify a Table

First, at the top of the file add comments to describe what the migration does. For example:

```rust
//! Creates a new table for storing users.
```

or 

```rust
//! Adds a new column to the users table.
```

Next, you should modify the enum at the end of the file

```rust
#[derive(DeriveIden)]
enum /* Table name (plural) */ {
    Table,
    Id,
    /* Add new columns here */
}
```

The `Table` variant should be the name of the table. The `Id` variant should be the name of the primary key column. The other variants should be the names of the columns in the table.

## Using the Migration

To apply the migration, run the following command:

```bash
sea-orm-cli migrate up -d ./crates/migrations/
```

Finaly, we will want to generate the code for the new table. To do this, run the following commands:

```bash
rm -rf ./crates/orm/src
sea-orm-cli generate entity --lib -o ./crates/orm/src
```

## Migration Backwards Compatibility

When making changes to the database schema, you must ensure that the changes are backwards compatible. This means that the new schema must be compatible with the old code. This allows us to have zero downtime deployments.

Migrating the database will normally be done in two steps. First, we will have a backwards compatible schema then we will remove the backwards compatibility after the new code is deployed.

### Users Table Example (Adding a Column)

Let's start with a schema

| Column Name | Type    | Nullable |
| ----------- | ------- | -------- |
| id          | INTEGER | ❌        |
| name        | TEXT    | ❌        |
| email       | TEXT    | ❌        |

Let's suppose we want to add a new column `avatar` to the table. We will first add the column as nullable. This will allow us to deploy the new code without having to migrate the database. The new schema will look like this:

| Column Name | Type    | Nullable |
| ----------- | ------- | -------- |
| id          | INTEGER | ❌        |
| name        | TEXT    | ❌        |
| email       | TEXT    | ❌        |
| avatar      | STRING  | ✅        |

This schema is backwards compatible because the new column is nullable. This means that the old code will still work with the new schema. Our new code will be able to set and get the value of the new column. The old code will just ignore the new column.

After the new code is deployed, we can migrate the database to make the new column non-nullable. This will make the new column required. The new schema will look like this:

| Column Name | Type    | Nullable |
| ----------- | ------- | -------- |
| id          | INTEGER | ❌        |
| name        | TEXT    | ❌        |
| email       | TEXT    | ❌        |
| avatar      | STRING  | ❌        |

This schema is not backwards compatible because the new column is not nullable. It's fine though because the new code is already deployed.

Congratulations! You have successfully migrated the database schema.

> Wait one sec, we now can't collect the email of the users anymore! What shall we do?

Let's do this agin!

Our goal is to remove the `email` column from the table. We will first make the column nullable. This will allow us to remove all code that uses the column. The new schema will look like this:

| Column Name | Type    | Nullable |
| ----------- | ------- | -------- |
| id          | INTEGER | ❌        |
| name        | TEXT    | ❌        |
| email       | TEXT    | ✅        |
| avatar      | STRING  | ❌        |

Now (again), we can deploy the new code without having to migrate the database. The new code will not use the `email` column. The old code will still use the `email` column.

After the new code is deployed, we can migrate the database to remove the `email` column. The new schema will look like this:

| Column Name | Type    | Nullable |
| ----------- | ------- | -------- |
| id          | INTEGER | ❌        |
| name        | TEXT    | ❌        |
| avatar      | STRING  | ❌        |

Yay! We did it again!

### GitLab Database Migrations

We recommend looking at the [GitLab Database Migrations](https://docs.gitlab.com/ee/development/database/) documentation for more information on how to make backwards compatible migrations. Please note, however, that not everything in the docs are aplicable to this project. 