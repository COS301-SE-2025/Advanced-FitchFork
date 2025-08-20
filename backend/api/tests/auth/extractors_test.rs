#[cfg(test)]
mod tests {
    use api::auth::claims::{AuthUser, Claims};
    use axum::{http::{Request, StatusCode}, extract::FromRequestParts};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::{env, time::{SystemTime, UNIX_EPOCH}};

    fn generate_token(claims: &Claims, secret: &str) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_valid_token() {
        unsafe {
            env::set_var("JWT_SECRET", "test_secret");
        }
        
        let claims = Claims {
            sub: 1,
            exp: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600) as usize,
            admin: true,
        };
        let token = generate_token(&claims, "test_secret");

        let request = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();
        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_ok());
        let auth_user = result.unwrap();
        assert_eq!(auth_user.0.sub, 1);
        assert_eq!(auth_user.0.admin, true);
    }

    #[tokio::test]
    async fn test_invalid_token_wrong_secret() {
        unsafe {
            env::set_var("JWT_SECRET", "test_secret");
        }

        let claims = Claims {
            sub: 1,
            exp: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600) as usize,
            admin: false,
        };
        let token = generate_token(&claims, "wrong_secret");

        let request = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();
        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Invalid or expired token");
    }

    #[tokio::test]
    async fn test_expired_token() {
        unsafe {
            env::set_var("JWT_SECRET", "test_secret");
        }

        let claims = Claims {
            sub: 2,
            exp: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 3600) as usize,
            admin: true,
        };
        let token = generate_token(&claims, "test_secret");

        let request = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();
        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Invalid or expired token");
    }

    #[tokio::test]
    async fn test_missing_authorization_header() {
        let request = Request::builder().body(()).unwrap();

        let (mut parts, _) = request.into_parts();
        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Missing or invalid Authorization header");
    }

    #[tokio::test]
    async fn test_malformed_authorization_header() {
        let request = Request::builder()
            .header("Authorization", "Bearer")
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();
        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(result.is_err());
        let (status, message) = result.unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Missing or invalid Authorization header");
    }
}