# Security Considerations

This document outlines important security considerations when using `axum-gate`.

## Password Security

- **Secure Hashing**: `axum-gate` uses Argon2 for password hashing, which includes automatic salt generation and is resistant to timing attacks.
- **No Plain Text Storage**: Passwords are never stored in plain text. Only the Argon2 hash is stored.
- **Unicode Support**: The system properly handles unicode passwords and special characters.

## JWT Token Security

- **Signature Validation**: All JWTs are cryptographically verified using the configured secret key.
- **Expiration Checking**: Tokens are automatically validated for expiration to prevent replay attacks.
- **Issuer Validation**: The system validates the issuer claim to prevent token substitution attacks.
- **Tamper Detection**: Any modification to the JWT will result in signature validation failure.

## Cookie Security

When using cookie-based authentication, ensure proper security attributes:

```rust
let cookie_template = axum_gate::cookie::CookieBuilder::new("auth-cookie", "")
    .secure(true)           // HTTPS only
    .http_only(true)        // Prevent XSS access
    .same_site(cookie::SameSite::Strict)  // CSRF protection
    .path("/")              // Limit scope
    .domain("example.com"); // Limit domain
```

## Authorization Security

- **Role Hierarchy**: The system properly enforces role hierarchies to prevent privilege escalation.
- **Permission Boundaries**: Fine-grained permissions are enforced at the route level.
- **Default Deny**: All routes are denied by default until explicitly granted access.

## Input Validation

- The system handles various input edge cases including:
  - Empty credentials
  - Extremely long inputs
  - Unicode characters
  - Special characters in usernames/passwords
  - Malformed JSON requests

## Storage Security

- **Isolation**: Different storage instances are properly isolated.
- **Thread Safety**: All storage operations are thread-safe for concurrent access.
- **Proper Cleanup**: Account and secret deletion is handled securely.

## Timing Attack Considerations

While the current implementation may have timing differences between existing and non-existing users during login attempts, consider implementing constant-time comparisons or artificial delays in production environments where timing attacks are a concern.

## Best Practices

1. **Use HTTPS**: Always use HTTPS in production to protect tokens in transit.
2. **Secure Secrets**: Use strong, randomly generated secrets for JWT signing.
3. **Regular Key Rotation**: Consider implementing key rotation for long-lived applications.
4. **Monitor Failed Attempts**: Implement rate limiting and monitoring for failed authentication attempts.
5. **Session Management**: Implement proper session invalidation on logout.
6. **Audit Logging**: Log authentication and authorization events for security monitoring.

## Security Testing

The crate includes comprehensive security tests covering:
- JWT manipulation attempts
- Password security edge cases
- Authorization bypass attempts
- Input validation against injection attacks
- Cookie security verification
- Storage layer security
- Edge cases and error conditions

Run the security tests with:
```bash
cargo test security_tests
cargo test authorization_security_tests  
cargo test storage_and_cookie_security_tests
```