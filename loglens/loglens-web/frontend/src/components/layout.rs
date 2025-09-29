use yew::prelude::*;
use crate::components::{Header, Sidebar};

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Children,
}

#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    let sidebar_open = use_state(|| false);
    
    let toggle_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_| {
            sidebar_open.set(!*sidebar_open);
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50">
            <Header {toggle_sidebar} />
            
            <div class="flex">
                <Sidebar open={*sidebar_open} />
                
                <main class={classes!(
                    "flex-1",
                    "transition-all",
                    "duration-300",
                    "lg:ml-64",
                    if *sidebar_open { "ml-64" } else { "ml-0" }
                )}>
                    <div class="p-6">
                        { for props.children.iter() }
                    </div>
                </main>
            </div>
        </div>
    }
}