use auth_middleware_for_actix_web::session::session_auth::UserSession;
use actix_web::{
    post,
    web::{Form, Data, ServiceConfig},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

use crate::domain::{
    auth_api::AuthenticationApi,
    user_api::UserApi,
};

#[derive(Serialize, Deserialize)]
struct FormLogin {
    email: String,
    password: String,
}

#[post("/login")]
async fn login(
    login_form: Form<FormLogin>,
    session: UserSession,
    user_api: Data<dyn UserApi>,
    auth_api: Data<dyn AuthenticationApi>,
) -> impl Responder {
    println!(
        "login request: email={}, password={}",
        login_form.email, login_form.password
    );

    match user_api.find_by_email(&login_form.email).await {
        Ok(user) => {
            if auth_api.is_password_correct(&user, &login_form.password).await {
                session.set_user(user).expect("User could not be set in session");
                return HttpResponse::Ok();
            } else {
                println!("{} tried to login but password was incorrect.", login_form.email)
            }
        }
        Err(_) => println!("{} tried to login but user not found.", login_form.email),
    }

    HttpResponse::BadRequest()
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(login);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use auth_middleware_for_actix_web::{middleware::{AuthMiddleware, PathMatcher}, session::session_auth::GetUserFromSession};

    use actix_web::{
        test::{call_service, init_service, TestRequest},
        web::Data,
        App,
    };

    use crate::{
        create_session_middleware,
        domain::{auth_api::{AuthToken, AuthenticationApi}, user::User, user_api::UserApi},
    };

    use super::{config, FormLogin};

    struct TestAuth;

    #[async_trait]
    impl AuthenticationApi for TestAuth {
        async fn is_password_correct(&self, _user: &crate::domain::user::User, password: &str) -> bool {
            password == "test123"
        }

        fn is_authenticated(&self, _auth: &dyn AuthToken) -> bool {
            true
        }

        fn get_authenticated_user(&self, _auth: &dyn AuthToken) -> Result<User, ()> {
            Ok(User::new(1, "test", "test"))
        }

    }

    struct TestUserService;
    #[async_trait]
    impl UserApi for TestUserService {
        async fn find_by_email(
            &self,
            _email: &str,
        ) -> Result<crate::domain::user::User, crate::error::errors::NotFoundError> {
            Ok(User::new(1, "test", "Test"))
        }
    }

    macro_rules! test_app_config {
        () => {{
            let user_api: Arc<dyn UserApi> = Arc::new(TestUserService);
            let auth_api: Arc<dyn AuthenticationApi> = Arc::new(TestAuth);
            let user_api_data = Data::from(user_api);
            let auth_api_data = Data::from(auth_api);


            let key = actix_web::cookie::Key::generate();
            App::new()
                .configure(config)
                .app_data(user_api_data.clone())
                .app_data(auth_api_data.clone())
                .wrap(AuthMiddleware::<_, User>::new(GetUserFromSession, PathMatcher::default()))
                .wrap(create_session_middleware(key.clone()))
        }};
    }

    #[actix_web::test]
    async fn should_response_ok_when_password_correct() {
        let config = test_app_config!();

        let app = init_service(config).await;

        let req = TestRequest::post()
            .uri("/login")
            .set_form(FormLogin {
                email: "test".to_owned(),
                password: "test123".to_owned(),
            })
            .to_request();
        let res = call_service(&app, req).await;

        assert!(res.status().is_success());
    }
}
