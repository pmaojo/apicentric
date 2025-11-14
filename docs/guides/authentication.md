# Authentication Guide

This guide explains how to use the authentication system in Apicentric's cloud server.

## Overview

Apicentric provides JWT-based authentication for securing the cloud API. Authentication can be optionally enabled based on your deployment needs.

## Features

- **JWT Token Authentication**: Secure token-based authentication
- **Token Refresh**: Refresh tokens without re-authenticating
- **Token Revocation**: Logout functionality with token blacklisting
- **Optional Authentication**: Enable/disable authentication via environment variables
- **Secure Password Storage**: Argon2 password hashing

## Environment Variables

### Authentication Configuration

```bash
# Enable/disable authentication (default: false)
APICENTRIC_PROTECT_SERVICES=true

# JWT secret key (change in production!)
APICENTRIC_JWT_SECRET=your-secret-key-here

# Database path for user storage
APICENTRIC_AUTH_DB=data/auth.db
```

### CORS Configuration

```bash
# Environment mode (development or production)
APICENTRIC_ENV=production

# Allowed origins for CORS (comma-separated)
ALLOWED_ORIGINS=https://example.com,https://app.example.com
```

## API Endpoints

### Register a New User

```bash
POST /api/auth/register
Content-Type: application/json

{
  "username": "myuser",
  "password": "mypassword123"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Login

```bash
POST /api/auth/login
Content-Type: application/json

{
  "username": "myuser",
  "password": "mypassword123"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Get Current User

```bash
GET /api/auth/me
Authorization: Bearer <token>
```

**Response:**
```json
{
  "username": "myuser"
}
```

### Refresh Token

Refresh your JWT token to extend the session without re-authenticating.

```bash
POST /api/auth/refresh
Authorization: Bearer <token>
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Note:** The old token is automatically blacklisted and cannot be reused.

### Logout

Invalidate your current token.

```bash
POST /api/auth/logout
Authorization: Bearer <token>
```

**Response:**
```json
{
  "message": "Successfully logged out",
  "success": true
}
```

## Using Authentication in Requests

Once you have a token, include it in the `Authorization` header for all protected endpoints:

```bash
curl -H "Authorization: Bearer <your-token>" \
  http://localhost:8000/api/services
```

## Error Responses

### Missing Authorization Header

```json
{
  "error": "Missing Authorization header",
  "code": "MISSING_AUTH_HEADER"
}
```

### Invalid Token Format

```json
{
  "error": "Invalid authorization scheme. Expected 'Bearer <token>'",
  "code": "INVALID_AUTH_SCHEME"
}
```

### Expired Token

```json
{
  "error": "Token has expired",
  "code": "TOKEN_EXPIRED"
}
```

### Revoked Token

```json
{
  "error": "Token has been revoked",
  "code": "TOKEN_REVOKED"
}
```

### Invalid Token

```json
{
  "error": "Invalid token format",
  "code": "INVALID_TOKEN"
}
```

## Security Best Practices

### Production Deployment

1. **Always set a strong JWT secret:**
   ```bash
   APICENTRIC_JWT_SECRET=$(openssl rand -base64 32)
   ```

2. **Enable authentication:**
   ```bash
   APICENTRIC_PROTECT_SERVICES=true
   ```

3. **Configure CORS properly:**
   ```bash
   APICENTRIC_ENV=production
   ALLOWED_ORIGINS=https://yourdomain.com
   ```

4. **Use HTTPS in production** to prevent token interception

5. **Rotate JWT secrets periodically** (requires re-authentication of all users)

### Token Lifecycle

- **Token Expiration**: Tokens expire after 24 hours
- **Token Refresh**: Use `/api/auth/refresh` to get a new token before expiration
- **Token Revocation**: Logout immediately blacklists the token
- **Blacklist Cleanup**: Blacklisted tokens are kept in memory until server restart

### Password Requirements

- Minimum length: 6 characters
- Passwords are hashed using Argon2
- No maximum length enforced (reasonable limits apply)

## Development Mode

For local development, you can disable authentication:

```bash
# .env file
APICENTRIC_PROTECT_SERVICES=false
APICENTRIC_ENV=development
```

This allows unrestricted access to all endpoints without tokens.

## Integration Examples

### JavaScript/TypeScript

```typescript
class ApiClient {
  private token: string | null = null;

  async login(username: string, password: string) {
    const response = await fetch('http://localhost:8000/api/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });
    const data = await response.json();
    this.token = data.token;
    return data;
  }

  async refreshToken() {
    const response = await fetch('http://localhost:8000/api/auth/refresh', {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${this.token}` },
    });
    const data = await response.json();
    this.token = data.token;
    return data;
  }

  async logout() {
    await fetch('http://localhost:8000/api/auth/logout', {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${this.token}` },
    });
    this.token = null;
  }

  async getServices() {
    const response = await fetch('http://localhost:8000/api/services', {
      headers: { 'Authorization': `Bearer ${this.token}` },
    });
    return response.json();
  }
}
```

### Python

```python
import requests

class ApiClient:
    def __init__(self, base_url='http://localhost:8000'):
        self.base_url = base_url
        self.token = None

    def login(self, username, password):
        response = requests.post(
            f'{self.base_url}/api/auth/login',
            json={'username': username, 'password': password}
        )
        data = response.json()
        self.token = data['token']
        return data

    def refresh_token(self):
        response = requests.post(
            f'{self.base_url}/api/auth/refresh',
            headers={'Authorization': f'Bearer {self.token}'}
        )
        data = response.json()
        self.token = data['token']
        return data

    def logout(self):
        requests.post(
            f'{self.base_url}/api/auth/logout',
            headers={'Authorization': f'Bearer {self.token}'}
        )
        self.token = None

    def get_services(self):
        response = requests.get(
            f'{self.base_url}/api/services',
            headers={'Authorization': f'Bearer {self.token}'}
        )
        return response.json()
```

## Troubleshooting

### "Token has been revoked" error

This occurs when:
- You've logged out and are trying to use the old token
- You've refreshed your token and are using the old one
- The server was restarted (blacklist is in-memory)

**Solution:** Login again to get a new token.

### "Token has expired" error

Tokens expire after 24 hours.

**Solution:** Use the `/api/auth/refresh` endpoint to get a new token, or login again.

### CORS errors in browser

If you see CORS errors in the browser console:

1. Check that `ALLOWED_ORIGINS` includes your frontend URL
2. Ensure `APICENTRIC_ENV` is set correctly
3. Verify the frontend is using the correct API URL

### Authentication not working

1. Verify `APICENTRIC_PROTECT_SERVICES=true` is set
2. Check that you're including the `Authorization` header
3. Ensure the token format is `Bearer <token>`
4. Verify the JWT secret hasn't changed

## Architecture

### Token Blacklist

The token blacklist is an in-memory data structure that stores hashed tokens. When a user logs out or refreshes their token, the old token is added to the blacklist.

**Limitations:**
- Blacklist is cleared on server restart
- Blacklist grows over time (until restart)
- No automatic cleanup of expired tokens

**Future Improvements:**
- Persistent blacklist storage
- Automatic cleanup of expired tokens
- Distributed blacklist for multi-instance deployments

### Middleware

The authentication middleware validates tokens on protected endpoints:

1. Extracts token from `Authorization` header
2. Checks if token is blacklisted
3. Validates token signature and expiration
4. Adds claims to request extensions for downstream handlers

Protected endpoints automatically receive validated user claims.

## See Also

- [Configuration Guide](configuration.md)
- [Error Handling Guide](error-handling.md)
- [Quick Start Guide](quick-start.md)
