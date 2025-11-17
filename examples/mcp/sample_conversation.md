# Sample MCP Conversation: Creating a Blog API

This example shows how an AI assistant uses MCP tools to create and manage a blog API service.

## User Request
"Create a blog API with posts and comments endpoints"

## AI Assistant Workflow (using MCP tools)

### Step 1: Analyze Requirements
The user wants:
- Blog posts (CRUD operations)
- Comments on posts
- RESTful API design

### Step 2: Create Service Definition
**AI uses `create_service` tool:**

```yaml
name: blog-api
version: "1.0"
description: Blog API with posts and comments
server:
  port: 9003
  base_path: /api/v1

fixtures:
  posts:
    - id: 1
      title: "Welcome to our Blog"
      content: "This is our first blog post..."
      author: "admin"
      created_at: "2025-01-01T10:00:00Z"
    - id: 2
      title: "Second Post"
      content: "Another interesting post..."
      author: "editor"
      created_at: "2025-01-02T14:30:00Z"

  comments:
    - id: 1
      post_id: 1
      content: "Great first post!"
      author: "reader1"
      created_at: "2025-01-01T11:00:00Z"

endpoints:
  # Posts endpoints
  - method: GET
    path: /posts
    response:
      status: 200
      content_type: application/json
      body: |
        {
          "posts": {{json fixtures.posts}},
          "total": {{fixtures.posts.length}}
        }

  - method: GET
    path: /posts/{id}
    response:
      status: 200
      content_type: application/json
      body: |
        {
          "post": {{find fixtures.posts "id" params.id}}
        }

  - method: POST
    path: /posts
    response:
      status: 201
      content_type: application/json
      body: |
        {
          "id": {{faker "datatype.number"}},
          "title": "{{request.body.title}}",
          "content": "{{request.body.content}}",
          "author": "{{request.body.author}}",
          "created_at": "{{now}}"
        }

  # Comments endpoints
  - method: GET
    path: /posts/{postId}/comments
    response:
      status: 200
      content_type: application/json
      body: |
        {
          "comments": {{json (filter fixtures.comments "post_id" params.postId)}}
        }

  - method: POST
    path: /posts/{postId}/comments
    response:
      status: 201
      content_type: application/json
      body: |
        {
          "id": {{faker "datatype.number"}},
          "post_id": {{params.postId}},
          "content": "{{request.body.content}}",
          "author": "{{request.body.author}}",
          "created_at": "{{now}}"
        }
```

### Step 3: Start the Service
**AI uses `start_service` tool:**
- Service starts on port 9003
- All endpoints become available

### Step 4: Verify Service Status
**AI uses `get_service_logs` tool:**
- Confirms service is running
- Shows successful startup logs

### Step 5: Provide Usage Examples
**AI responds to user:**

✅ **Blog API Created Successfully!**

Your blog API is now running on `http://localhost:9003/api/v1`

**Available endpoints:**

**Posts:**
- `GET /api/v1/posts` - List all posts
- `GET /api/v1/posts/{id}` - Get specific post
- `POST /api/v1/posts` - Create new post

**Comments:**
- `GET /api/v1/posts/{postId}/comments` - Get comments for a post
- `POST /api/v1/posts/{postId}/comments` - Add comment to post

**Test it:**

```bash
# Get all posts
curl http://localhost:9007/api/v1/posts

# Create a new post
curl -X POST http://localhost:9007/api/v1/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My New Post",
    "content": "This is the content...",
    "author": "me"
  }'

# Get comments for post #1
curl http://localhost:9007/api/v1/posts/1/comments
```

The API includes realistic data with timestamps, dynamic IDs, and proper JSON responses. You can now integrate this with your frontend or use it for testing!

## Technical Details

- **Port:** 9003
- **Base Path:** `/api/v1`
- **Data:** Mock blog posts and comments with realistic content
- **Features:** Dynamic ID generation, timestamp handling, request validation
- **Format:** Standard REST API with JSON responses

This demonstrates how MCP enables AI assistants to create complete, functional API services from natural language descriptions in seconds!