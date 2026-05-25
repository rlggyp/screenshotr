# screenshotr

**screenshotr** is a lightweight web screenshot service built in Rust, utilizing [fantoccini](https://github.com/jonhoo/fantoccini) to automate browser interactions via WebDriver. It is designed for secure, authenticated, and controlled screenshot capture of web pages.

## Features

* Captures screenshots of web pages using a headless browser (WebDriver)
* Basic authentication support
* HMAC-SHA256 signature verification for request integrity
* Domain whitelisting to mitigate SSRF (Server-Side Request Forgery) attacks
* Dockerized for easy deployment

## API Endpoints

### `/api/screenshotr`

**Method:** `POST`  
**Description:** Capture a screenshot of the specified URL.

**Request Body:**
```json
{
  "url": "https://rlggyp.com"
}
```

**Responses:**

- **200 OK**  
  Returns a JSON object with the image URL.
  ```json
  {
    "image_url": "http://localhost:12009/screenshotr/images/123e4567-e89b-12d3-a456-426614174000.png"
  }
  ```

- **400 Bad Request**  
  Returned if the request body is invalid.
  ```json
  {
    "error": "Invalid request body"
  }
  ```

- **403 Forbidden**  
  Returned if the requested URL is not in the allowed domains.
  ```json
  {
    "error": "URL not allowed"
  }
  ```

- **500 Internal Server Error**  
  Returned if an internal error occurs during screenshot capture.
  ```json
  {
    "error": "error details"
  }
  ```

### `/screenshotr/images/{filename}`

**Method:** `GET`  
**Description:** Retrieve a screenshot image by filename.

**Example:**
```
GET /screenshotr/images/123e4567-e89b-12d3-a456-426614174000.png
```

## Configuration

### Example `config.yaml`

```yaml
hmac_secret: helloworldrlggyp
basic_auth_users:
  rlggyp: $2a$12$ZHe/gc5ZMJggDpK7OD0OMeqvDpdwxtk8Jx587FGg1qVaPETwloV2K
screenshot:
  page_load_delay_secs: 2
  public_base_url: http://localhost:12009
  webdriver_url: http://localhost:4444
  webdriver_capabilities:
    browserName: chrome
    goog:chromeOptions:
      args:
      - --headless
      - --no-sandbox
      - --disable-dev-shm-usage
      - --disable-gpu
      - --window-size=1920,1200
allowed_domains:
  - rlggyp.com
replay_protection:
  nonce_ttl_secs: 60
  max_nonce_cache_size: 1000
```

- `hmac_secret`: Secret key for HMAC-SHA256 signature verification.
- `basic_auth_users`: Map of usernames to bcrypt-hashed passwords.
- `screenshot`: Screenshot capture settings.
- `allowed_domains`: List of allowed domains for screenshot requests (SSRF protection).
- `replay_protection`: Replay attack prevention settings.
  - `nonce_ttl_secs`: Time-to-live for nonce in seconds (default: 60).
  - `max_nonce_cache_size`: Maximum number of nonces stored in cache (default: 1000).

### Example `log4rs.yaml`

```yaml
refresh_rate: 30 seconds

appenders:
  rolling_file:
    kind: rolling_file
    path: "/etc/screenshotr/logs/screenshotr.log"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 1 mb
      roller:
        kind: fixed_window
        pattern: "/etc/screenshotr/logs/screenshotr-{}.log"
        base: 1
        count: 7
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)} [{l}] {t} - {m}{n}"

root:
  level: info
  appenders:
    - rolling_file
```

## Docker Usage

### Docker Compose Example

Below is a sample `docker-compose.yaml`:

```yaml
services:
  screenshotr:
    image: rlggyp/screenshotr:latest
    container_name: screenshotr
    user: 1000:1000
    environment:
      - CONFIG_FILE=/etc/screenshotr/configs/config.yaml
      - LOG_CONFIG_FILE=/etc/screenshotr/configs/log4rs.yaml
    ports:
      - 12009:12009
    volumes:
      - ./assets:/assets
      - ./configs:/etc/screenshotr/configs
      - ./logs:/etc/screenshotr/logs
    restart: unless-stopped
```

**Directory structure example:**
```
.
├── assets
├── configs
│   ├── config.yaml
│   └── log4rs.yaml
├── logs
└── docker-compose.yaml
```

- Place your `config.yaml` and `log4rs.yaml` in the configs directory.
- Screenshots will be saved in the assets directory.

### Running with Docker Compose

1. Build or pull the Docker image.
2. Adjust your configuration files as needed.
3. Start the service:

    ```bash
    docker compose up -d
    ```

## Client Authentication

Requests must be authenticated using Basic Auth and HMAC-SHA256 signature verification with replay attack protection.

### Request Headers Required
- `Authorization`: Basic authentication (base64 encoded `username:password`)
- `Signature-256`: HMAC-SHA256 signature with format `sha256={hex_encoded_signature}`
- `Timestamp`: Unix timestamp in seconds (UTC)
- `Nonce`: Unique identifier (UUID v4 recommended)

### Signature Calculation

The signature is calculated using a canonical message format:
```
canonical_message = "{timestamp}\n{nonce}\n{sha256(body)}"
signature = hex(HMAC-SHA256(secret, canonical_message))
```

### Python Example

```python
import requests
import hmac
import hashlib
import json
import uuid
from datetime import datetime, timezone
import base64

# Configuration
SERVER_URL = "http://localhost:12009"
HMAC_SECRET = "helloworldrlggyp"
USERNAME = "rlggyp"
PASSWORD = "your-password"

def make_authenticated_request(url_to_screenshot):
    # Prepare request body
    payload = json.dumps({"url": url_to_screenshot})
    body_bytes = payload.encode()
    
    # Generate nonce and timestamp
    nonce = str(uuid.uuid4())
    timestamp = str(int(datetime.now(timezone.utc).timestamp()))

    
    # Calculate signature
    body_hash = hashlib.sha256(body_bytes).hexdigest()
    canonical_message = f"{timestamp}\n{nonce}\n{body_hash}"
    signature = hmac.new(
        HMAC_SECRET.encode(),
        canonical_message.encode(),
        hashlib.sha256
    ).hexdigest()
    
    # Prepare headers
    auth_string = base64.b64encode(
        f"{USERNAME}:{PASSWORD}".encode()
    ).decode()
    
    headers = {
        "Authorization": f"Basic {auth_string}",
        "Signature-256": f"sha256={signature}",
        "Timestamp": timestamp,
        "Nonce": nonce,
        "Content-Type": "application/json"
    }
    
    # Make request
    response = requests.post(
        f"{SERVER_URL}/api/screenshotr",
        data=body_bytes,
        headers=headers
    )
    
    return response.json()

# Usage
try:
    result = make_authenticated_request("https://rlggyp.com")
    print(f"Screenshot URL: {result['image_url']}")
except Exception as e:
    print(f"Error: {e}")
```

### Notes
- Each request must have a **unique nonce** - reusing a nonce within the TTL will be rejected as a replay attack
- The **timestamp** must be within the TTL window (default 60 seconds) of server time
- Request body must match exactly the bytes used in signature calculation
- All components (timestamp, nonce, body) are protected by the signature

## Security Notes

- Only domains listed in `allowed_domains` can be used for screenshot requests, preventing SSRF attacks.
- Use strong secrets and passwords for authentication and HMAC.
- Replay attack protection prevents the same request from being accepted multiple times.
- Always use HTTPS in production to prevent credential interception.

## Limitations

- Only supports HTTP/HTTPS URLs.
- Only domains in the whitelist are allowed.
- Requires a running WebDriver (e.g., ChromeDriver or geckodriver).