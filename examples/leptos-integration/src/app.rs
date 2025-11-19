use crate::protected::ProtectedView;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// For route generation only.
#[component]
pub fn UnprotectedRoutesForRouteGeneration() -> impl IntoView {
    view! {
        <Router>
            <UnprotectedRoutes />
        </Router>
    }
}

/// For route generation only.
#[component]
pub fn ProtectedRoutesForRouteGeneration() -> impl IntoView {
    view! {
        <Router>
            <ProtectedRoutes />
        </Router>
    }
}

/// Needs to be called within a `<Router>` component.
#[component]
pub fn UnprotectedRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| "Page not found.".into_view()>
        <Route path=StaticSegment("") view=HomePage/>
        </Routes>
    }
}

/// Needs to be called within a `<Router>` component.
#[component]
pub fn ProtectedRoutes() -> impl IntoView {
    view! {
        <Routes fallback=|| "Page not found.".into_view()>
        <Route path=StaticSegment("/protected") view=ProtectedView/>
        </Routes>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/axum-gate-leptos.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <UnprotectedRoutes />
                <ProtectedRoutes />
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}
