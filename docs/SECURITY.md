# VividShift Security Guide

## Overview
Comprehensive security implementation guide for VividShift Generic Assignment Engine covering authentication, authorization, data protection, and security best practices.

**Target Audience:** Security engineers, system administrators, developers

## Authentication System

### JWT Token Management
VividShift uses JWT (JSON Web Tokens) for stateless authentication:

```rust
// Token structure
{
  "sub": "user_id",
  "username": "admin", 
  "role": "admin",
  "exp": 1640995200,
  "iat": 1640908800
}
```

### Token Security Features
- **Expiration:** Configurable token lifetime (default: 24 hours)
- **Signing:** HMAC-SHA256 with secure secret key
- **Revocation:** Database-tracked session management
- **Refresh:** Automatic token renewal before expiration

### Password Security
- **Hashing:** bcrypt with configurable cost factor (default: 12)
- **Validation:** Minimum 8 characters, complexity requirements
- **Storage:** Never stored in plaintext
- **Rotation:** Forced password changes for compromised accounts

## Authorization Framework

### Role-Based Access Control (RBAC)
```rust
pub enum UserRole {
    Admin,    // Full system access
    User,     // Standard operations
    Viewer,   // Read-only access
}
```

### Permission Matrix
| Resource | Admin | User | Viewer |
|----------|-------|------|--------|
| Users | CRUD | Read (self) | None |
| Participants | CRUD | CRUD | Read |
| Assignments | CRUD | CRUD | Read |
| System Config | CRUD | None | None |

### API Endpoint Protection
```rust
// Middleware-based protection
#[middleware(require_auth)]
#[middleware(require_role("admin"))]
async fn admin_endpoint() -> Result<Response> {
    // Admin-only functionality
}
```

## Data Protection

### Database Security
- **Connection Encryption:** SSL/TLS for all database connections
- **Parameterized Queries:** Prevents SQL injection attacks
- **Access Control:** Database user with minimal required privileges
- **Audit Logging:** Complete change tracking with user attribution

### Sensitive Data Handling
```sql
-- Password hashing
CREATE TABLE users (
    password_hash VARCHAR(255) NOT NULL  -- bcrypt hashed
);

-- Session management
CREATE TABLE user_sessions (
    token_hash VARCHAR(255) NOT NULL,   -- SHA-256 hashed
    expires_at TIMESTAMPTZ NOT NULL
);
```

### Data Encryption
- **At Rest:** Database-level encryption for sensitive fields
- **In Transit:** HTTPS/TLS for all API communications
- **Application Level:** Sensitive configuration encrypted

## Network Security

### HTTPS Configuration
```nginx
server {
    listen 443 ssl http2;
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains";
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
}
```

### Rate Limiting
```toml
[security.rate_limiting]
requests_per_minute = 60
burst_size = 10
auth_requests_per_minute = 5
```

### CORS Configuration
```rust
let cors = CorsLayer::new()
    .allow_origin("https://yourdomain.com".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

## Security Monitoring

### Audit Logging
```rust
// Security event logging
audit_log::record_event(AuditEvent {
    user_id: Some(user.id),
    action: "login_attempt",
    resource: "authentication",
    success: true,
    ip_address: client_ip,
    user_agent: user_agent,
    timestamp: Utc::now(),
});
```

### Intrusion Detection
- **Failed Login Monitoring:** Track and alert on suspicious login patterns
- **Rate Limit Violations:** Monitor and block excessive requests
- **Unusual Access Patterns:** Detect abnormal user behavior
- **SQL Injection Attempts:** Log and alert on malicious queries

### Security Metrics
```rust
// Prometheus security metrics
counter!("auth_attempts_total", &[("result", "success")]);
counter!("auth_attempts_total", &[("result", "failure")]);
histogram!("session_duration_seconds");
gauge!("active_sessions_count");
```

## Vulnerability Management

### Input Validation
```rust
use validator::Validate;

#[derive(Validate)]
pub struct CreateUser {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}
```

### SQL Injection Prevention
```rust
// Always use parameterized queries
sqlx::query!(
    "SELECT * FROM users WHERE username = $1",
    username
)
.fetch_one(&pool)
.await?;
```

### XSS Protection
- **Content Security Policy:** Strict CSP headers
- **Input Sanitization:** All user input validated and sanitized
- **Output Encoding:** Proper encoding for all dynamic content

## Security Configuration

### Production Security Settings
```toml
[security]
jwt_secret = "use-strong-random-secret-in-production"
bcrypt_cost = 14
session_timeout = 3600
require_https = true
secure_cookies = true

[security.headers]
hsts_enabled = true
csp_enabled = true
frame_options = "DENY"
content_type_options = "nosniff"
```

### Environment Variables
```bash
# Security-critical environment variables
VIVIDSHIFT_AUTH_JWT_SECRET=your-256-bit-secret
VIVIDSHIFT_DATABASE_SSL_MODE=require
VIVIDSHIFT_SECURITY_REQUIRE_HTTPS=true
```

## Incident Response

### Security Incident Procedures
1. **Detection:** Monitor logs and alerts for security events
2. **Assessment:** Evaluate severity and impact
3. **Containment:** Isolate affected systems
4. **Eradication:** Remove threats and vulnerabilities
5. **Recovery:** Restore normal operations
6. **Lessons Learned:** Document and improve security measures

### Emergency Response
```bash
# Revoke all user sessions
cargo run --bin security_cli revoke-all-sessions

# Disable user account
cargo run --bin security_cli disable-user --username=compromised_user

# Force password reset
cargo run --bin security_cli force-password-reset --user-id=uuid
```

## Compliance and Standards

### Security Standards Compliance
- **OWASP Top 10:** Protection against common web vulnerabilities
- **NIST Cybersecurity Framework:** Comprehensive security controls
- **ISO 27001:** Information security management standards

### Data Privacy
- **GDPR Compliance:** User data protection and privacy rights
- **Data Minimization:** Collect only necessary information
- **Right to Erasure:** User data deletion capabilities
- **Data Portability:** Export user data in standard formats

## Security Testing

### Automated Security Testing
```bash
# Dependency vulnerability scanning
cargo audit

# Static code analysis
cargo clippy -- -W clippy::all

# Security-focused tests
cargo test security::
```

### Penetration Testing
- **Authentication Bypass:** Test for authentication vulnerabilities
- **Authorization Flaws:** Verify access control implementation
- **Input Validation:** Test for injection vulnerabilities
- **Session Management:** Validate session security

## References
- [Configuration Guide](CONFIGURATION.md) - Security configuration options
- [API Reference](API_REFERENCE.md) - Authentication and authorization
- [Deployment Guide](DEPLOYMENT.md) - Production security setup
- [OWASP Security Guidelines](https://owasp.org/) - Web application security
