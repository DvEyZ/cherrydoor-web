[Back](/)

# POST /auth
Attempts to obtain an authentication token.

## Request

### Request body
Entity defined by the JSON [schema](/schemas/auth/auth.request.schema.json).

~~~~json
{
    "name": "some-user",
    "password": "some-password"
}
~~~~

## Response

### Status codes
- `200 OK`, if the authentication succeeds.
- `403 Forbidden`, if the authentication fails.

### Response body
Entity defined by the JSON [schema](/schemas/auth/auth.response.schema.json).

~~~~json
{
    "token": "some-auth-token-in-JWT-format"
}
~~~~

# Using the token

The token should be posted as `Bearer` authorization token in `Authorization` header whenever making a request to an URL requiring authorization.

```
Authorization: Bearer some-auth-token-in-JWT-format
```