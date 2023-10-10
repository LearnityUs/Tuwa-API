# Enviorment Variables

The below is a example `.env` file. You will need to create a `.env` file in the root of the project. The `.env` file is used to set environment variables. The `.env` file is not checked into version control. The `.env` file is used by the `dotenv` crate to set environment variables when running the application.
```bash
# What level of logging to use.
RUST_LOG=DEBUG
# The port to run the server on.
PORT=8080
# The URL to the database.
DATABASE_URL=postgres://<username>:<password>@<host>:<port>/<database>
# The CORS origin to allow. For development you can just put `*`.
CORS_ORIGIN=*
# The Schoology consumer key and secret.
SCHOOLOGY_CONSUMER_KEY=key
# The Schoology consumer secret.
SCHOOLOGY_CONSUMER_SECRET=secret
```

## Required

The following are required to run the application.

`DATABASE_URL` - The URL to the database. The URL should be in the format `postgres://<username>:<password>@<host>:<port>/<database>`
`SCHOOLOGY_CONSUMER_KEY` - The Schoology consumer key.
`SCHOOLOGY_CONSUMER_SECRET` - The Schoology consumer secret.

## Optional

The following are optional.

`RUST_LOG` - The level of logging to use. The default is `OFF`. The levels are `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE`.
`PORT` - The port to run the server on. The default is `8080`.
`CORS_ORIGIN` - The CORS origin to allow. For development you can just put `*`. The default is `(null)` disallowing all origins.