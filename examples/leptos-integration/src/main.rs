#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_gate::prelude::*;
    use axum_gate_leptos::app::*;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use std::sync::Arc;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Initialize and configure your Gate
    let secret = "DEV_STRING".to_string();
    let options = JsonWebTokenOptions {
        enc_key: axum_gate::jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        dec_key: axum_gate::jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        header: None,
        validation: None,
    };
    let jwt = Arc::new(JsonWebToken::<JwtClaims<Account<Role, Group>>>::new_with_options(options));
    let gate = Gate::cookie::<_, Role, Group>("my-app", jwt.clone())
        .allow_anonymous_with_optional_user()
        .configure_cookie_template(|tpl| tpl.name("auth-token"))
        .unwrap();
    let admin_gate = Gate::cookie::<_, Role, Group>("my-app", jwt)
        .with_policy(AccessPolicy::require_role(Role::Admin))
        .configure_cookie_template(|tpl| tpl.name("auth-token"))
        .unwrap();

    let routes = generate_route_list(App);
    // The following is an attempt to split the route generation into protected and unprotected
    // routes. It fails because it tries to add the "user" server function twice.
    //
    // Generate the list of PROTECTED routes in your Leptos App
    // let protected_routes = generate_route_list(ProtectedRoutesForRouteGeneration);
    // Generate the list of UNprotected routes in your Leptos App
    // let unprotected_routes = generate_route_list(UnprotectedRoutesForRouteGeneration);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        // .leptos_routes(&leptos_options, protected_routes, {
        //     let leptos_options = leptos_options.clone();
        //     move || shell(leptos_options.clone())
        // })
        // .layer(admin_gate)
        // .leptos_routes(&leptos_options, unprotected_routes, {
        //     let leptos_options = leptos_options.clone();
        //     move || shell(leptos_options.clone())
        // })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(gate)
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
