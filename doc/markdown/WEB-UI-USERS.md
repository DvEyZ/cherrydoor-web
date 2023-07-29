[Back](/)

# GET /web-ui-users
Lists all Web UI users.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.

### Response body
An array of entities defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.full.schema.json).

```json
[
    {
        "name": "some-user",
        "is_admin": false
    },
    {
        "name": "other-user",
        "is_admin": true
    }
]
```

# POST /web-ui-users
Creates new Web UI user.

## Request

### Authorization
Requires authorized Web UI user with admin permissions (`is_admin == true`)


### Request Body
Entity defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.create.schema.json).

```json
{
    "name": "some-name",
    "password": "some-password",
    "is_admin": false
}
```

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `409 Conflict`, if the user with the provided `name` already exists.

### Response body
Entity defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.full.schema.json).

# GET /web-ui-users/&lt;name&gt;
Gets a single Web UI user.

## Request

### Authorization
Requires authorized Web UI user.

### URL parameters
- `name` - an username of the web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
Entity defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.full.schema.json).

# PATCH /web-ui-users/&lt;name&gt;
Updates an existing Web UI user.

## Request

### Authorization
Requires authorized Web UI user with admin permissions (`is_admin == true`)

### URL parameters
- `name` - an username of the web UI user.

### Request body
Entity defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.update.schema.json).

```json
{
    "password": "new-password",
    "is_admin": true
}
```

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
Entity defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.full.schema.json).

# DELETE /web-ui-users/&lt;name&gt;
Deletes a Web UI user.

## Request

### Authorization
Requires authorized Web UI user with admin permissions (`is_admin == true`)

### URL parameters
- `name` - an username of the web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
Entity defined by the JSON [schema](/schemas/web-ui-users/web-ui-user.full.schema.json).