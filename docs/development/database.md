# Making Changes to the Database

This document describes how to make changes to the database schema.

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

Next, we will want to generate the code for the new table. To do this, run the following commands:

```bash
rm -rf ./crates/migrations/src
sea-orm-cli generate entity --lib -o ./crates/orm/src
```