# `/api/v1/schoology/user` - GET

This endpoint fetches the user data from the schoology API. This endpoint requires the user to be authenticated with `user` permissions.

## Response Body

### RouteError

This endpoint will return a `RouteError` if the request is unsuccessful. The `data` field will be a enum representation of the error.
 - SchoologyNotLinked: `Client Fault` - This error is returned when the user has not linked their schoology account or the token has expired.
 - DatabaseError: `Server Fault` - This is a generic error that is returned when the database returns an error that is not handled by the API.

```json
{
    "type": "RouteError",
    "data": "SchoologyNotLinked"
}
```

### Success

This endpoint will return a `Success` if the request is successful. The `data` field will be a object with the following fields:
 - `first_name`: `string` - The user's first name.
 - `last_name`: `string` - The user's last name.
 - `picture_url`: `string` - The url to the user's profile picture.

```json
{
    "type": "Success",
    "data": {
        "first_name": "John",
        "last_name": "Doe",
        "picture_url": "https://example.com/picture.jpg"
    }
}
```