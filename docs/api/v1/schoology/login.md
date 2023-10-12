# `/api/v1/schoology/login` - POST

This endpoint is used to login to schoology and does not require any authentication. The request body should be a json object with the following fields:
 - `id`: `string` - The uuid gotten from `/api/v1/schoology/request_uuid`
 - `signature`: `string` - The signature gotten from `/api/v1/schoology/request_token`
 - `login`: `boolean` - Weather or not to create a new session.

## Request Body

```json
{
    "id": "string",
    "signature": "string",
    "login": "boolean"
}
```

## Response Body

### RouteError

This endpoint will return a `RouteError` if the request is unsuccessful. The `data` field will be a enum representation of the error.
 - SchoologyError: `Server Fault` - This is a generic error that is returned when schoology returns an error that is not handled by the API.
 - DatabaseError: `Server Fault` - This is a generic error that is returned when the database returns an error that is not handled by the API.
 - InvalidFlowId: `Client Fault` - This is returned when the id returned from `/api/v1/schoology/request_token` is invalid / expired.
 - InvalidSignature: `Client Fault` - This is returned when the signature does not match the id returned from `/api/v1/schoology/request_token`.
 - ApplicationNotAuthorized: `Client Fault` - This is returned when the application is not authorized to access the user's schoology account.

```json
{
    "type": "RouteError",
    "data": "SchoologyError"
}
```

### Success

This endpoint will return a `Success` if the request is successful. The `data` field will either be a an empty object *(If `login` is `false`)* or a object with the following fields *(If `login` is `true`)*:
 - `session_token`: `string` - The session token to use for future requests.
 - `session_expires_at`: `string` - The time at which the session will expire. `2023-10-10T00:00:00.000000Z` This is in ISO 8601 format.

```json
{
    "type": "Success",
    "data": {
        "session_token": "string",
        "session_expires_at": "string"
    }
}
```