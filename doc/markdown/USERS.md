[Back](/)

# GET /users
Lists all users.

## Request

### Authorization
Requires authorized Web UI user.

### URL params
- `page` (integer, optional) - page number, starting from `0`. Defaults to `0`. Pagination was implemented for this route because it is likely to have a large number of resources. Page size is `10`.

## Response

### Status codes
- `200 OK`, if the request succeeds.

### Response body
An array of entities defined by the JSON [schema](/schemas/users/user.brief.schema.json).

```json
[
    {
        "id": 1,
        "name": "john-doe",
        "full_name": "John F. Doe",
        "role": "Member of the Department of Something Unimportant"
    },
    {
        "id": 1,
        "name": "jane-doe",
        "full_name": "Jane Q. Doe",
        "role": "Member of the Department of Something Unimportant"
    }
]
```

# POST /users
Creates a new user.

## Request

### Authorization
Requires authorized Web UI user.

### Request body
An entity defined by the JSON [schema](/schemas/users/user.create.schema.json).

```json
{
    "name": "john-doe",
    "full_name": "John F. Doe",
    "role": "A new user."
}
```

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `409 Conflict`, if an user with the provided `name` already exists.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

```json
{
    "id": 1,
    "name": "john-doe",
    "full_name": "John F. Doe",
    "role": "A new user.",
    "access_codes": [],
    "permissions": []
}
```

# GET /users/&lt;name&gt;
Gets a single user.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

# PATCH /users/&lt;name&gt;
Updates an user.

## Request

### Authorization
Requires authorized Web UI user.

### Request body
An entity defined by the JSON [schema](/schemas/users/user.update.schema.json).

```json
{
    "full_name": "John Ichangedmylastname",
    "role": "Member of the Department of Something Even Less Important"
}
```

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

# DELETE /users/&lt;name&gt;
Deletes an user.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

# GET /users/&lt;name&gt;/access-codes
Lists all access codes registered for an user.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
An array of entities defined by the JSON [schema](/schemas/users/access-codes/access-code.user.full.schema.json).

```json
[
    {
        "id": 1,
        "code": "123456789",
        "user": 1
    },
    {
        "id": 2,
        "code": "321098765",
        "user": 1
    }
]
```

# GET /users/&lt;name&gt;/access-codes/&lt;id&gt;
Gets one access code.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist, or an access code with the id `id` does not exist, or is not registered for this user.

### Response body
An entity defined by the JSON [schema](/schemas/users/access-codes/access-code.user.full.schema.json).

# POST /users/&lt;name&gt;/access-codes
Manually registers an access code.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

```json
{
    "id": 1,
    "name": "john-doe",
    "full_name": "John F. Doe",
    "role": "A new user.",
    "access_codes": [
        {
            "id": 1,
            "code": "123456789",
            "user": 1
        }
    ],
    "permissions": []
}
```

# POST /users/&lt;name&gt;/access-codes/register

### ⚠️ Work in progress

# DELETE /users/&lt;name&gt;/access-codes/&lt;id&gt;
Deletes an access code.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist, or an access code with the id `id` does not exist, or is not registered for this user.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

# GET /users/&lt;name&gt;/permissions
Lists permissions the user does possess.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist, or an access code with the id `id` does not exist, or is not registered for this user.

### Response body
An array of entities defined by the JSON [schema](/schemas/permissions/permission.brief.schema.json).

```json
[
    {
        "id": 1,
        "name": "morning",
        "description": "This permission grants access from 5:00 to 8:00"
    },
    {
        "id": 2,
        "name": "night",
        "description": "This permission grants access from 22:00 to 6:00"
    }
]
```

# POST /users/&lt;name&gt;/permissions
Assigns a new permission for the user.

## Request

### Authorization
Requires authorized Web UI user.

### Request body
An entity defined by the JSON [schema](/schemas/users/permissions/permission.user.create.schema.json).

```json
{
    "permission_id": 2
}
```

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist, or an permission with the id `permission_id` does not exist.
- `409 Conflict`, if the user has this permission already assigned.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).

```json
{
    "id": 1,
    "name": "john-doe",
    "full_name": "John F. Doe",
    "role": "A new user.",
    "access_codes": [],
    "permissions": [
        {
            "id": 2,
            "name": "night",
            "description": "This permission grants access from 22:00 to 6:00"
        }
    ]
}
```
# DELETE /users/&lt;name&gt;/permissions/&lt;id&gt;
Revoke a permission for the user.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.
- `404 Not Found`, if the user with the provided `name` does not exist, or an permission with the id `id` does not exist or is not assigned to this user.

### Response body
An entity defined by the JSON [schema](/schemas/users/user.full.schema.json).