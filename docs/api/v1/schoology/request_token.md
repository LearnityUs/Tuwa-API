# `/api/v1/schoology/request_token` - GET

This endpoint makes a request to the schoology `/oauth/request_token` endpoint and returns the id and signature to use for the login request. This endpoint does not require any authentication.

## Response Body

### RouteError

This endpoint will return a `RouteError` if the request is unsuccessful. The `data` field will be a enum representation of the error.
 - SchoologyError: `Server Fault` - This is a generic error that is returned when schoology returns an error that is not handled by the API.
 - DatabaseError: `Server Fault` - This is a generic error that is returned when the database returns an error that is not handled by the API.

```json
{
    "type": "RouteError",
    "data": "SchoologyError"
}
```

### Success

This endpoint will return a `Success` if the request is successful. The `data` field will be a object with the following fields:
 - `id`: `string` - The uuid to use for the login request.
 - `signature`: `string` - The signature to use for the login request.
 - `expires_at`: `string` - The time at which the id and signature will expire. `2023-10-10T00:00:00.000000Z` This is in ISO 8601 format.

```json
{
    "type": "Success",
    "data": {
        "id": "string",
        "signature": "string"
    }
}
```