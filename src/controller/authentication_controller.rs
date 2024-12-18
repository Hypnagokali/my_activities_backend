use std::time::{Duration, SystemTime};

use actix_session::Session;
use actix_web::{
    post,
    web::{self, Data},
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
    login_form: web::Form<FormLogin>,
    session: Session,
    user_api: Data<dyn UserApi>,
    auth_api: Data<dyn AuthenticationApi>,
) -> impl Responder {
    println!(
        "login request: email={}, password={}",
        login_form.email, login_form.password
    );

    match user_api.find_by_email(&login_form.email) {
        Ok(user) => {
            if auth_api.is_password_correct(&user, &login_form.password) {
                session
                    .insert("user", user)
                    .expect("Cant create session key 'user'");
                let now = SystemTime::now();
                let ttl = now + Duration::from_secs(30 * 60);

                session
                    .insert("ttl", ttl)
                    .expect("Cant create session key 'ttl'")
            }
        }
        Err(_) => return HttpResponse::Unauthorized(),
    }

    HttpResponse::Ok()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{
        test::{call_service, init_service, TestRequest},
        web::Data,
        App,
    };

    use crate::{
        application::authentication::AuthMiddleware,
        create_session_middleware,
        domain::{auth_api::{AuthToken, AuthenticationApi}, user::User, user_api::UserApi},
    };

    use super::{config, FormLogin};

    struct TestAuth;

    impl AuthenticationApi for TestAuth {
        fn is_password_correct(&self, _user: &crate::domain::user::User, password: &str) -> bool {
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
    impl UserApi for TestUserService {
        fn find_by_email(
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

            App::new()
                .configure(config)
                .app_data(user_api_data.clone())
                .app_data(auth_api_data.clone())
                .wrap(AuthMiddleware::new())
                .wrap(create_session_middleware())
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
