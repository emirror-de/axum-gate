use axum_gate::prelude::{Account, Group, Role};
use leptos::prelude::*;

#[server(endpoint = "user")]
pub async fn user() -> Result<Option<Account<Role, Group>>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::{extract, redirect};

    let Extension(Some(user)): Extension<Option<Account<Role, Group>>> = extract().await? else {
        redirect("/");
        return Ok(None);
    };
    Ok(Some(user))
}

/// Protected view.
#[component]
pub fn ProtectedView() -> impl IntoView {
    let user_info = LocalResource::new(move || async move { user().await.unwrap() });
    let logged_in_info = move || match user_info.get() {
        Some(u) => format!("{u:?}"),
        None => "Loading...".to_string(),
    };
    view! {
        { logged_in_info }
    }
}
