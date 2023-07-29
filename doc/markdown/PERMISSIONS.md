[Back](/)

# GET /permissions
Lists all permissions.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `200 OK`, if the request succeeds.

### Response body
An array of entities defined by the JSON [schema](/schemas/permissions/permission.brief.schema.json)

```json
[
    {
        "id": 1,
        "name": "morning",
        "description": "This permission grants access from 6:00 to 8:00."
    },
    {
        "id": 2,
        "name": "night",
        "description": "This permission grants access from 22:00 to 3:00"
    }
]
```

# POST /permissions
Creates a new permission.

## Request

### Authorization
Requires authorized Web UI user.

### Request body
An entity defined by the JSON [schema](/schemas/permissions/permission.create.schema.json)

```json
{
    "name": "morning",
    "description": "This permission grants access from 6:00 to 8:00."
}
```

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `409 Conflict`, if a permission with the provided `name` already exists.

### Response body
An entity defined by the JSON [schema](/schemas/permissions/permission.full.schema.json)

```json
{
    "id": 1,
    "name": "morning",
    "description": "This permission grants access from 6:00 to 8:00.",
    "users": [],
    "access_profiles": []
}
```

# GET /permissions/&lt;name&gt;
Gets a single permission.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `404 Not Found`, if a permission with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/permissions/permission.full.schema.json)

# PATCH /permissions/&lt;name&gt;
Updates a permission.

## Request

### Authorization
Requires authorized Web UI user.

### Request body
An entity defined by the JSON [schema](/schemas/permissions/permission.update.schema.json)

```json
{
    "description": "some new description"
}
```

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `404 Not Found`, if a permission with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/permissions/permission.full.schema.json)

# DELETE /permissions/&lt;name&gt;
Deletes a permission.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `404 Not Found`, if a permission with the provided `name` does not exist.

### Response body
An entity defined by the JSON [schema](/schemas/permissions/permission.full.schema.json)

# GET /permissions/&lt;name&gt;/access-profiles
List access profiles for which the permission grants access.

## Request

### Authorization
Requires authorized Web UI user.

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `404 Not Found`, if a permission with the provided `name` does not exist.

### Response body
An array of entities defined by the JSON [schema](/schemas/permissions/access-profiles/access-profile.permission.full.schema.json)

```json
[
    {
        "id": 1,
        "name": "night",
        "description": "Active from 23:00 to 5:00"
    },
    {
        "id": 2,
        "name": "day",
        "description": "Active from 6:00 to 21:00"
    }
]
```

# POST /permissions/&lt;name&gt;/access-profiles
Assign an access to access profile for the permission.

## Request

### Authorization
Requires authorized Web UI user.

### Request body
An entity defined by the JSON [schema](/schemas/permissions/access-profiles/access-profile.permission.create.schema.json)

```json
{
    "access_profile_id": 1
}
```

## Response

### Status codes
- `201 Created`, if the request succeeds.
- `404 Not Found`, if a permission with the provided `name` or an access profile with provided `access-profile-id` does not exist.
- `409 Conflict`, if an access profile with provided `access-profile-id` is already assigned to the permission.

### Response body
An entity defined by the JSON [schema](/schemas/permissions/permission.full.schema.json)

```json
{
    "id": 1,
    "name": "morning",
    "description": "This permission grants access from 6:00 to 8:00.",
    "users": [],
    "access_profiles": [
        {
            "id": 1,
            "name": "night",
            "description": "Active from 23:00 to 5:00"
        }
    ]
}
```