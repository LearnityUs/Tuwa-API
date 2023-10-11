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

```json
{
    "type": "success",
    "data": {
        "id": 1,
        "name": "John Doe"
    }
}
```