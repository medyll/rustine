use dioxus::prelude::*;

pub fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            h1 { "Rustine — prototype" }
            p { "This is a minimal Dioxus UI scaffold." }
            // TODO: implement UrlInput, UrlList, UrlItem components
        }
    })
}
