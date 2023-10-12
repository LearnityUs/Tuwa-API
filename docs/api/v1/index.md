# V1 API

Currently this is the latest version of the API.

## Authentication

The v1 API uses bearer token authentication. This will be passed in the `Authorization` header of the request. 

```http
GET /api/v1/this_endpoint_does_not_exist HTTP/1.1
Authorization: Bearer <token>
```

## Response Format

There are 3 types of responses that the API will return:
 - `Success` - The request was successful and the data is returned.
 - `RouteError` - The request was unsuccessful due to an error generated in the endpoint. (Some exeptions)
 - `RequestError` - The request was unsuccessful due to an error pre-route. (Maliformed json, missing auth, etc)

### Success

This is the standard response for a successful request. Note that the `data` will very accross endpoints.

```json
{
    "type": "Success",
    "data": {
        "id": 1,
        "name": "John Doe"
    }
}
```

These results will always return a status code of `200`.

### RouteError


This is the standard response for a request that was unsuccessful due to an error generated in the endpoint. Note that the `data` will very accross endpoints. Generaly, howver, the `data` will be a enum representation of the error.

```json
{
    "type": "RouteError",
    "data": "UnauthorizedError"
}
```

Note that the route will return the status codes `400` or `500` depending on the error. `400` is used for errors that are the user's fault (Bad request, etc) and `500` is used for errors that are the server's fault (Internal server error, etc).

### RequestError

This is a standardized response for a request that was unsuccessful due to an error pre-route. (Maliformed json, missing auth, etc)

```json
{
    "type": "RequestError",
    "status": "NotFound"
}
```

Here are all the possible `status` values:
 - `NotFound` - The endpoint does not exist OR the requested resource was not found. (Used only when a path parameter is used).
 - `Unauthorized` - The user is not authenticated.
 - `Forbidden` - The user is authenticated, but does not have the required credentials.
 - `BadRequest` - The request was malformed.
 - `InternalServerError` - The server encountered an internal error.

Note that the HTTP status code will be set to the corresponding value.