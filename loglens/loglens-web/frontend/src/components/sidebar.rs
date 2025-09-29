use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub open: bool,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let current_route = use_route::<Route>();

    let nav_items = vec![
        ("Dashboard", Route::Dashboard, "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"),
        ("Projects", Route::Projects, "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"),
        ("Settings", Route::Settings, "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"),
    ];

    html! {
        <aside class={classes!(
            "sidebar",
            "fixed",
            "left-0",
            "top-16",
            "z-40",
            "w-64",
            "h-screen",
            "bg-white",
            "border-r",
            "border-gray-200",
            "transition-transform",
            "duration-300",
            "lg:translate-x-0",
            if props.open { "translate-x-0" } else { "-translate-x-full" }
        )}>
            <div class="flex flex-col h-full">
                <nav class="flex-1 px-4 py-6 space-y-1">
                    {for nav_items.iter().map(|(name, route, icon_path)| {
                        let is_active = current_route.as_ref() == Some(route);
                        
                        html! {
                            <Link<Route>
                                to={route.clone()}
                                classes={classes!(
                                    "flex",
                                    "items-center",
                                    "px-3",
                                    "py-2",
                                    "text-sm",
                                    "font-medium",
                                    "rounded-md",
                                    "transition-colors",
                                    "duration-150",
                                    if is_active {
                                        "bg-blue-100 text-blue-700"
                                    } else {
                                        "text-gray-700 hover:bg-gray-100 hover:text-gray-900"
                                    }
                                )}
                            >
                                <svg class="mr-3 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={*icon_path} />
                                </svg>
                                {*name}
                            </Link<Route>>
                        }
                    })}
                </nav>

                <div class="px-4 py-4 border-t border-gray-200">
                    <div class="text-xs text-gray-500 text-center">
                        {"LogLens v0.1.0"}
                    </div>
                </div>
            </div>
        </aside>
    }
}